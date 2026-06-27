use axum::extract::Path;
use chrono::NaiveDateTime;
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::{env, time};
use tokio::fs;
use tower_http::services::ServeDir;

use axum::{
    Json, Router,
    extract::{FromRequestParts, State},
    http::{StatusCode, Uri, header},
    response::{IntoResponse, Response},
    routing::{delete, get, patch, post},
};
use mime_guess;

#[derive(RustEmbed)]
#[folder = "ui/dist"]
struct Asset;

#[derive(Clone)]
struct AppState {
    info: Arc<Mutex<serde_json::Value>>,
    pool: SqlitePool,
}

#[derive(Deserialize, Serialize)]
struct NoteRequest {
    summary: String,
}

#[derive(Deserialize, Serialize)]
struct NoteStatusRequest {
    is_completed: bool,
}

#[derive(Deserialize, Serialize, sqlx::FromRow)]
struct NoteResponse {
    id: i32,
    summary: String,
    is_completed: bool,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

struct ValidToken;

impl<S> FromRequestParts<S> for ValidToken
where
    S: Send + Sync,
{
    type Rejection = StatusCode;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .ok_or(StatusCode::UNPROCESSABLE_ENTITY)?;
        let secret = env::var("PROFILE_TOKEN").map_err(|_| StatusCode::UNAUTHORIZED)?;
        if secret == token.to_string() {
            Ok(ValidToken)
        } else {
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let content = fs::read_to_string("./profile.json").await.unwrap();
    let json_content = serde_json::from_str::<serde_json::Value>(&content).unwrap();
    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .min_connections(5)
        .acquire_timeout(time::Duration::from_secs(5))
        .connect("./database.sqlite")
        .await
        .unwrap();
    let mut tx = pool.begin().await.unwrap();
    sqlx::query(
        "
        CREATE TABLE IF NOT EXISTS notes(
            id INTEGER PRIMARY KEY,
            summary TEXT NOT NULL,
            is_completed INTEGER DEFAULT 0,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    )",
    )
    .execute(&mut *tx)
    .await
    .unwrap();
    sqlx::query(
        "
        CREATE INDEX IF NOT EXISTS idx_notes
        ON notes(id);
    ",
    )
    .execute(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();

    let appstate = AppState {
        info: Arc::new(Mutex::new(json_content)),
        pool: pool,
    };

    let info_routes = Router::new()
        .route("/get", get(get_info))
        .route("/update", post(update_info));
    let notes_routes = Router::new()
        .route("/get", get(get_notes))
        .route("/create", post(create_note))
        .route("/update/{id}", patch(update_note))
        .route("/update-status/{id}", patch(update_note_status))
        .route("/delete/{id}", delete(delete_note));
    let api_routes = Router::new()
        .nest("/info", info_routes)
        .nest("/notes", notes_routes);
    let app = Router::new()
        .nest_service("/assets", ServeDir::new("assets"))
        .nest("/api", api_routes)
        .with_state(appstate)
        .fallback(serve_embedded_file);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8787));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!(
        "🚀 Server is running at http://0.0.0.0:{}",
        listener.local_addr().unwrap().port()
    );

    axum::serve(listener, app).await.unwrap();
}

async fn serve_embedded_file(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');

    match Asset::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, mime.as_ref())],
                content.data,
            )
                .into_response()
        }
        None => match Asset::get("index.html") {
            Some(content) => (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "text/html")],
                content.data,
            )
                .into_response(),
            None => StatusCode::NOT_FOUND.into_response(),
        },
    }
}

async fn get_info(State(state): State<AppState>) -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"data": state.info})))
}

async fn update_info(State(state): State<AppState>) -> impl IntoResponse {
    let content = fs::read_to_string("./profile.json").await.unwrap();
    let json_content = serde_json::from_str::<serde_json::Value>(&content).unwrap();
    let mut data = match state.info.lock() {
        Ok(data) => data,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "message": "Something went wrong!"
                })),
            );
        }
    };
    *data = json_content.clone();
    (
        StatusCode::OK,
        Json(json!({
            "data": json_content
        })),
    )
}

async fn get_notes(State(state): State<AppState>) -> impl IntoResponse {
    let records = match sqlx::query_as::<_, NoteResponse>("SELECT * FROM notes")
        .fetch_all(&state.pool)
        .await
    {
        Ok(records) => records,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "message": "Something went wrong!"
                })),
            );
        }
    };
    (
        StatusCode::OK,
        Json(json!({
            "data": records
        })),
    )
}

async fn create_note(
    ValidToken: ValidToken,
    State(state): State<AppState>,
    Json(payload): Json<NoteRequest>,
) -> impl IntoResponse {
    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "message": "Something went wrong!"
                })),
            );
        }
    };
    if sqlx::query("INSERT INTO notes (summary) VALUES (?)")
        .bind(&payload.summary)
        .execute(&mut *tx)
        .await
        .is_err()
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "message": "Something went wrong!"
            })),
        );
    }
    if tx.commit().await.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "message": "Something went wrong!"
            })),
        );
    }
    (
        StatusCode::OK,
        Json(json!({
            "message": "Notes created"
        })),
    )
}

async fn update_note(
    ValidToken: ValidToken,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<NoteRequest>,
) -> impl IntoResponse {
    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "message": "Something went wrong!"
                })),
            );
        }
    };
    if sqlx::query("UPDATE notes SET summary = ? WHERE id = ?")
        .bind(&payload.summary)
        .bind(&id)
        .execute(&mut *tx)
        .await
        .is_err()
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "message": "Something went wrong!"
            })),
        );
    }
    if tx.commit().await.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "message": "Something went wrong!"
            })),
        );
    }
    (
        StatusCode::CREATED,
        Json(json!({
            "message": "Notes updated!"
        })),
    )
}

async fn update_note_status(
    ValidToken: ValidToken,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<NoteStatusRequest>,
) -> impl IntoResponse {
    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "message": "Something went wrong!"
                })),
            );
        }
    };
    if sqlx::query("UPDATE notes SET is_completed = ? WHERE id = ?")
        .bind(&payload.is_completed)
        .bind(&id)
        .execute(&mut *tx)
        .await
        .is_err()
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "message": "Something went wrong!"
            })),
        );
    }
    if tx.commit().await.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "message": "Something went wrong!"
            })),
        );
    }
    (
        StatusCode::OK,
        Json(json!({
            "message": "Notes status updated!"
        })),
    )
}

async fn delete_note(
    ValidToken: ValidToken,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "message": "Something went wrong!"
                })),
            );
        }
    };
    if sqlx::query("DELETE FROM notes WHERE id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await
        .is_err()
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "message": "Something went wrong!"
            })),
        );
    }
    if tx.commit().await.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "message": "Something went wrong!"
            })),
        );
    }
    (
        StatusCode::OK,
        Json(json!({
            "message": "Notes deleted"
        })),
    )
}
