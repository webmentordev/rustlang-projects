use axum::{
    extract::{Path, State},
    response::{
        sse::{Event, KeepAlive, Sse},
        Html, IntoResponse,
    },
    routing::get,
    Json, Router,
};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    convert::Infallible,
    net::SocketAddr,
    path::PathBuf,
    process::Stdio,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    sync::broadcast,
};
use tokio_stream::wrappers::BroadcastStream;
use uuid::Uuid;

#[derive(Clone, Serialize, Debug)]
struct Progress {
    status: String,
    percent: f32,
    message: String,
}

#[derive(Clone)]
struct Job {
    app_id: String,
    seq: u64,
    tx: broadcast::Sender<Progress>,
    latest: Arc<Mutex<Progress>>,
}

type Jobs = Arc<Mutex<HashMap<String, Job>>>;

#[derive(Clone)]
struct AppState {
    jobs: Jobs,
    next_seq: Arc<AtomicU64>,
}

#[derive(Serialize)]
struct JobSummary {
    id: String,
    app_id: String,
    status: String,
    percent: f32,
    message: String,
}

#[derive(Deserialize)]
struct StartRequest {
    app_id: String,
    install_dir: String,
    anonymous: Option<bool>,
    username: Option<String>,
    password: Option<String>,
}

#[derive(Serialize)]
struct StartResponse {
    id: String,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        jobs: Arc::new(Mutex::new(HashMap::new())),
        next_seq: Arc::new(AtomicU64::new(0)),
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/api/jobs", get(list_jobs).post(start_download))
        .route("/api/jobs/:id/events", get(progress_stream))
        .with_state(state);

    let address = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect(format!("failed to bind to 0.0.0.0:{}", address.port()).as_str());
    println!("steam-dl listening on http://0.0.0.0:{}", address.port());
    axum::serve(listener, app).await.expect("server error");
}

async fn index() -> Html<&'static str> {
    Html(INDEX_HTML)
}

async fn start_download(
    State(state): State<AppState>,
    Json(req): Json<StartRequest>,
) -> Json<StartResponse> {
    let id = Uuid::new_v4().to_string();
    let (tx, _rx) = broadcast::channel(256);
    let latest = Arc::new(Mutex::new(Progress {
        status: "starting".into(),
        percent: 0.0,
        message: "Launching SteamCMD...".into(),
    }));

    let job = Job {
        app_id: req.app_id.clone(),
        seq: state.next_seq.fetch_add(1, Ordering::SeqCst),
        tx: tx.clone(),
        latest: latest.clone(),
    };
    state.jobs.lock().unwrap().insert(id.clone(), job);

    tokio::spawn(async move {
        run_steamcmd(req, tx, latest).await;
    });

    Json(StartResponse { id })
}

async fn list_jobs(State(state): State<AppState>) -> Json<Vec<JobSummary>> {
    let jobs = state.jobs.lock().unwrap();
    let mut out: Vec<JobSummary> = jobs
        .iter()
        .map(|(id, job)| {
            let p = job.latest.lock().unwrap().clone();
            JobSummary {
                id: id.clone(),
                app_id: job.app_id.clone(),
                status: p.status,
                percent: p.percent,
                message: p.message,
            }
        })
        .collect();
    // oldest first; the frontend prepends each one, so the newest job ends up on top
    let seqs: HashMap<&str, u64> = jobs.iter().map(|(id, j)| (id.as_str(), j.seq)).collect();
    out.sort_by_key(|s| seqs.get(s.id.as_str()).copied().unwrap_or(0));
    Json(out)
}

fn resolve_steamcmd() -> (PathBuf, Option<PathBuf>) {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            for candidate in ["steamcmd.sh", "steamcmd"] {
                let path = dir.join(candidate);
                if path.is_file() {
                    return (path, Some(dir.to_path_buf()));
                }
            }
        }
    }
    (PathBuf::from("steamcmd"), None)
}

async fn run_steamcmd(
    req: StartRequest,
    tx: broadcast::Sender<Progress>,
    latest: Arc<Mutex<Progress>>,
) {
    let (steamcmd_path, working_dir) = resolve_steamcmd();
    let mut cmd = Command::new(&steamcmd_path);
    if let Some(dir) = &working_dir {
        cmd.current_dir(dir);
    }
    cmd.arg("+@sSteamCmdForcePlatformType").arg("windows");
    let install_dir = {
        let p = PathBuf::from(&req.install_dir);
        if p.is_absolute() {
            p
        } else {
            std::env::current_dir().map(|cwd| cwd.join(&p)).unwrap_or(p)
        }
    };
    cmd.arg("+force_install_dir").arg(&install_dir);

    let username = req.username.clone().unwrap_or_default();
    if req.anonymous.unwrap_or(true) || username.is_empty() {
        cmd.arg("+login").arg("anonymous");
    } else {
        cmd.arg("+login")
            .arg(username)
            .arg(req.password.clone().unwrap_or_default());
    }

    cmd.arg("+app_update").arg(&req.app_id).arg("validate");
    cmd.arg("+quit");
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::null());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            publish(
                &tx,
                &latest,
                Progress {
                    status: "error".into(),
                    percent: 0.0,
                    message: format!(
                        "Couldn't launch {} ({e}). Expected it either next to this binary or on PATH.",
                                     steamcmd_path.display()
                    ),
                },
            );
            return;
        }
    };

    if let Some(stdout) = child.stdout.take() {
        let mut lines = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let mut p = latest.lock().unwrap().clone();

            if let Some(pct) = parse_percent(&line) {
                p.percent = pct;
                p.status = "downloading".into();
            }
            if line.contains("Success! App") {
                p.status = "completed".into();
                p.percent = 100.0;
            } else if line.to_lowercase().contains("error") {
                p.status = "error".into();
            }
            p.message = line;
            publish(&tx, &latest, p);
        }
    }

    let exit = child.wait().await;
    let mut p = latest.lock().unwrap().clone();
    if p.status != "completed" && p.status != "error" {
        match exit {
            Ok(s) if s.success() => {
                p.status = "completed".into();
                p.percent = 100.0;
                p.message = "Done.".into();
            }
            _ => {
                p.status = "error".into();
                p.message = "SteamCMD exited unexpectedly.".into();
            }
        }
        publish(&tx, &latest, p);
    }
}

fn publish(tx: &broadcast::Sender<Progress>, latest: &Arc<Mutex<Progress>>, p: Progress) {
    *latest.lock().unwrap() = p.clone();
    let _ = tx.send(p);
}

fn parse_percent(line: &str) -> Option<f32> {
    let idx = line.find("progress: ")?;
    let rest = &line[idx + "progress: ".len()..];
    let end = rest
        .find(|c: char| c != '.' && !c.is_ascii_digit())
        .unwrap_or(rest.len());
    if end == 0 {
        return None;
    }
    rest[..end].parse::<f32>().ok()
}

async fn progress_stream(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let job = state.jobs.lock().unwrap().get(&id).cloned();

    let Some(job) = job else {
        let stream = tokio_stream::once(Ok::<_, Infallible>(
            Event::default().event("error").data("unknown job id"),
        ));
        return Sse::new(stream).into_response();
    };

    let initial = job.latest.lock().unwrap().clone();
    let rx_stream =
        BroadcastStream::new(job.tx.subscribe()).filter_map(|res| async move { res.ok() });
    let combined = tokio_stream::once(initial).chain(rx_stream).map(|p| {
        let json = serde_json::to_string(&p).unwrap_or_default();
        Ok::<_, Infallible>(Event::default().data(json))
    });

    Sse::new(combined)
        .keep_alive(KeepAlive::default())
        .into_response()
}

const INDEX_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>steam-dl</title>
<style>
:root { color-scheme: dark; }
body {
font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
background: #121212; color: #e8e8e8;
max-width: 640px; margin: 40px auto; padding: 0 20px;
}
h1 { font-size: 1.4rem; margin-bottom: 4px; }
p.sub { color: #999; margin-top: 0; font-size: 0.9rem; }
form {
    background: #1c1c1c; border: 1px solid #2c2c2c; border-radius: 10px;
    padding: 16px; display: grid; gap: 10px; margin-bottom: 24px;
    }
    label { font-size: 0.85rem; color: #bbb; }
    input[type=text], input[type=password] {
    width: 100%; padding: 8px 10px; border-radius: 6px; border: 1px solid #333;
    background: #101010; color: #eee; box-sizing: border-box;
    }
    .row { display: flex; gap: 10px; align-items: center; }
    button {
    background: #2a6df4; color: white; border: none; padding: 10px 16px;
    border-radius: 6px; font-weight: 600; cursor: pointer;
    }
    button:hover { background: #1f5bd6; }
    #jobs { display: grid; gap: 12px; }
    .job {
    background: #1c1c1c; border: 1px solid #2c2c2c; border-radius: 10px; padding: 12px 14px;
    }
    .job .top { display: flex; justify-content: space-between; font-size: 0.85rem; color: #aaa; }
    .bar-track { background: #2c2c2c; border-radius: 6px; height: 10px; margin: 8px 0; overflow: hidden; }
    .bar-fill { background: #2a6df4; height: 100%; width: 0%; transition: width 0.3s ease; }
    .job.completed .bar-fill { background: #35b26a; }
    .job.error .bar-fill { background: #d9534f; }
    .msg { font-size: 0.78rem; color: #888; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
    </style>
    </head>
    <body>

    <h1>steam-dl</h1>
    <p class="sub">Downloads the Windows build of a Steam game onto this Linux machine via SteamCMD.</p>

    <form id="form">
    <div>
    <label>App ID</label>
    <input type="text" id="appId" placeholder="e.g. 730" required>
    </div>
    <div>
    <label>Install directory (on this machine)</label>
    <input type="text" id="installDir" placeholder="/home/user/games/mygame" required>
    </div>
    <div class="row">
    <input type="checkbox" id="anonymous" checked>
    <label for="anonymous">Anonymous login (only works for free-to-play / no-DRM apps)</label>
    </div>
    <div id="creds" style="display:none">
    <label>Username</label>
    <input type="text" id="username">
    <label>Password</label>
    <input type="password" id="password">
    </div>
    <button type="submit">Start download</button>
    </form>

    <div id="jobs"></div>

    <script>
    const form = document.getElementById('form');
    const jobsEl = document.getElementById('jobs');
    const anon = document.getElementById('anonymous');
    const creds = document.getElementById('creds');

    anon.addEventListener('change', () => {
    creds.style.display = anon.checked ? 'none' : 'block';
    });

    form.addEventListener('submit', async (e) => {
        e.preventDefault();
        const body = {
        app_id: document.getElementById('appId').value.trim(),
        install_dir: document.getElementById('installDir').value.trim(),
        anonymous: anon.checked,
username: document.getElementById('username').value,
password: document.getElementById('password').value,
};

const res = await fetch('/api/jobs', {
method: 'POST',
headers: { 'Content-Type': 'application/json' },
body: JSON.stringify(body),
});
const { id } = await res.json();
addJobRow(id, body.app_id);
});

function addJobRow(id, appId, initial) {
const el = document.createElement('div');
el.className = 'job' + (initial ? ' ' + initial.status : '');
el.id = 'job-' + id;
const pct = initial ? Math.round(initial.percent) : 0;
const msg = initial ? initial.message : 'Starting…';
el.innerHTML = `
<div class="top"><span>App ${appId}</span><span class="pct">${pct}%</span></div>
<div class="bar-track"><div class="bar-fill" style="width:${pct}%"></div></div>
<div class="msg">${msg}</div>
`;
jobsEl.prepend(el);

if (initial && (initial.status === 'completed' || initial.status === 'error')) {
    return; // finished before refresh — no need to reconnect
    }

    const source = new EventSource('/api/jobs/' + id + '/events');
    source.onmessage = (evt) => {
    const p = JSON.parse(evt.data);
    el.className = 'job ' + p.status;
    el.querySelector('.pct').textContent = Math.round(p.percent) + '%';
    el.querySelector('.bar-fill').style.width = p.percent + '%';
    el.querySelector('.msg').textContent = p.message;
    if (p.status === 'completed' || p.status === 'error') {
        source.close();
        }
        };
        source.onerror = () => source.close();
        }

        // On load, re-sync with whatever's still running (or finished) on the server —
        // downloads keep going server-side across a refresh, only the UI needed catching up.
        window.addEventListener('DOMContentLoaded', async () => {
        try {
        const res = await fetch('/api/jobs');
        const jobs = await res.json();
        for (const j of jobs) {
            addJobRow(j.id, j.app_id, j);
            }
            } catch (e) {
            console.error('Failed to load existing jobs', e);
            }
            });
            </script>

            </body>
            </html>
            "##;
