use actix_web::{App, HttpResponse, HttpServer, Responder, get, HttpRequest, web};
use actix_web_ratelimit::RateLimit;
use actix_web_ratelimit::config::RateLimitConfig;
use actix_web_ratelimit::store::MemoryStore;
use actix_files::NamedFile;
use std::sync::Arc;
use std::io::Result;
use std::collections::HashSet;
use uuid::Uuid;
use rand::Rng;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use mime_guess::from_path;
use rust_embed::RustEmbed;
use std::time::Duration;
use tokio::time::interval;
use std::path::Path;
use std::fs;


#[derive(RustEmbed)]
#[folder = "ui/dist"]
struct Asset;

#[actix_web::main]
async fn main() -> Result<()> {
    let port = 8888;
    println!("Listening at: http://127.0.0.1:{}", port);

    tokio::spawn(async {
        cleanup_old_files().await;
    });

    let config = RateLimitConfig::default().max_requests(5).window_secs(60);
    let store = Arc::new(MemoryStore::new());

    HttpServer::new(move || {
        App::new()
            .service(
                web::scope("/api")
                    .wrap(RateLimit::new(config.clone(), store.clone()))
                    .service(generate)
            )
            .service(download_file)
            .default_service(web::route().to(static_files))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}

#[get("/")]
async fn index() -> Result<impl Responder>{
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "All good!"
    })))
}

#[get("/generate/{total}")]
async fn generate(params: web::Path<usize>) -> Result<impl Responder>{
    let total = params.into_inner();
    let mut uuids = HashSet::with_capacity(total);
    for _ in 0..total {
        uuids.insert(Uuid::new_v4().to_string());
    }
    tokio::fs::create_dir_all("files").await?;
    let filename = format!("file-{}-{}.json", total, random_number(8));
    let fullpath = format!("files/{}", filename);
    let mut file = File::create(&fullpath).await?;
    let content= serde_json::json!({
        "file": filename,
        "success": true,
        "total_uuids": uuids.len(),
        "version": "4",
        "uuids": uuids
    });
    let final_format = serde_json::to_string_pretty(&content)?;
    file.write_all(final_format.as_bytes()).await?;
    Ok(HttpResponse::Ok().json(content))
}

#[get("/get-file/{file}")]
async fn download_file(params: web::Path<String>) -> Result<NamedFile> {
    let filename = params.into_inner();
    if filename.contains("..") || filename.contains("/") || filename.contains("\\") {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid filename"
        ));
    }
    let filepath = format!("files/{}", filename);
    NamedFile::open(filepath)
}

async fn static_files(req: HttpRequest) -> HttpResponse {
    let path = req.path().trim_start_matches('/');
    let file_path = if path.is_empty() { "index.html" } else { path };

    match Asset::get(file_path) {
        Some(content) => {
            let body = content.data.into_owned();
            let mime = from_path(file_path).first_or_octet_stream();
            HttpResponse::Ok().content_type(mime.as_ref()).body(body)
        }
        None => match Asset::get("index.html") {
            Some(index_file) => HttpResponse::Ok()
                .content_type("text/html")
                .body(index_file.data.into_owned()),
            None => HttpResponse::NotFound().body("404 Not Found"),
        },
    }
}

fn random_number(len: usize) -> String{
    const CHARS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let mut rng = rand::rng();
    let rand_nums: String = (0..len).map(|_|{
        let idx = rng.random_range(0..CHARS.len());
        CHARS[idx] as char
    }).collect();
    rand_nums
}

async fn cleanup_old_files() {
    let mut cleanup_interval = interval(Duration::from_secs(300));
    loop {
        cleanup_interval.tick().await;
        let files_dir = Path::new("files");
        if !files_dir.exists() {
            continue;
        }
        let now = std::time::SystemTime::now();
        if let Ok(entries) = fs::read_dir(files_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(age) = now.duration_since(modified) {
                            // Delete files older than 10 minutes
                            if age > Duration::from_secs(600) {
                                let path = entry.path();
                                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                                    if let Err(e) = fs::remove_file(&path) {
                                        eprintln!("Failed to delete {:?}: {}", path, e);
                                    } else {
                                        println!("Deleted old file: {:?}", path);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}