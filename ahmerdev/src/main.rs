use chrono::NaiveDateTime;
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::{env, time};
use tokio::fs;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

use axum::{
    Json, Router,
    extract::{FromRequestParts, Path, Query, State},
    http::{StatusCode, Uri, header},
    response::{IntoResponse, Response},
    routing::{delete, get, patch, post},
};
use mime_guess;

#[derive(RustEmbed)]
#[folder = "ui/.output/public"]
struct Asset;

#[derive(Clone)]
struct AppState {
    info: Arc<Mutex<serde_json::Value>>,
    pool: SqlitePool,
}

#[derive(Deserialize, Serialize)]
struct NoteRequest {
    summary: String,
    is_completed: Option<bool>,
    info_type: Option<String>,
}

#[derive(Deserialize, Serialize, sqlx::FromRow)]
struct NoteResponse {
    id: i32,
    summary: String,
    is_completed: bool,
    info_type: String,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

#[derive(Deserialize, Serialize)]
struct ProfessionalTodoRequest {
    todo: String,
    is_completed: Option<bool>,
    status: Option<String>,
}

#[derive(Deserialize, Serialize, sqlx::FromRow)]
struct ProfessionalTodoResponse {
    id: i32,
    todo: String,
    is_completed: bool,
    completed_at: Option<String>,
    status: String,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct DateRange {
    pub start: Option<chrono::NaiveDate>,
    pub end: Option<chrono::NaiveDate>,
}

#[derive(Deserialize, Debug)]
struct Pagination {
    page: Option<i32>, // Always passed so no need for Option!
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

    let sql_query = include_str!("../profile.example.json");
    ensure_profile_exists(&sql_query).await;

    let database_name = String::from("database.sqlite");
    let content = fs::read_to_string("./profile.json").await.unwrap();
    let json_content = serde_json::from_str::<serde_json::Value>(&content).unwrap();

    if !std::fs::exists(&database_name).unwrap() {
        println!("Database does not exist!");
        if let Err(_) = std::fs::File::create(&database_name) {
            println!("Failed to create the database file: {}", &database_name);
            return;
        } else {
            println!("Database has been created ✔️");
        }
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .min_connections(5)
        .acquire_timeout(time::Duration::from_secs(5))
        .connect(&database_name)
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

    let column_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('notes') WHERE name = 'info_type'",
    )
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    if column_exists == 0 {
        sqlx::query("ALTER TABLE notes ADD COLUMN info_type VARCHAR(255) NOT NULL DEFAULT 'task'")
            .execute(&mut *tx)
            .await
            .unwrap();
    }
    sqlx::query(
        "
        CREATE TABLE IF NOT EXISTS professional_todos(
            id INTEGER PRIMARY KEY,
            todo TEXT NOT NULL,
            is_completed INTEGER NOT NULL DEFAULT 0,
            completed_at DATETIME,
            status TEXT NOT NULL DEFAULT 'pending'
                CHECK (status IN ('pending', 'in_progress', 'completed')),
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();

    let appstate = AppState {
        info: Arc::new(Mutex::new(json_content)),
        pool: pool,
    };

    let cors = CorsLayer::permissive();
    let info_routes = Router::new()
        .route("/get", get(get_info))
        .route("/update", post(update_info));
    let notes_routes = Router::new()
        .route("/get", get(get_notes))
        .route("/create", post(create_note))
        .route("/update/{id}", patch(update_note))
        .route("/delete/{id}", delete(delete_note));
    let professional_todos_routes = Router::new()
        .route("/get", get(get_professional_todos))
        .route("/get/{id}", get(get_professional_todo))
        .route("/create", post(create_professional_todo))
        .route("/update/{id}", patch(update_professional_todo))
        .route("/delete/{id}", delete(delete_professional_todo));
    let api_routes = Router::new()
        .nest("/info", info_routes)
        .nest("/notes", notes_routes)
        .nest("/professional_todos", professional_todos_routes);
    let app = Router::new()
        .nest_service("/assets", ServeDir::new("assets"))
        .nest("/api", api_routes)
        .with_state(appstate)
        .layer(cors)
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

async fn get_notes(
    State(state): State<AppState>,
    Query(query): Query<Pagination>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1);
    let total_records: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM notes")
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);
    let record_per_page: i64 = 20;
    let total_pages = ((total_records as f64) / (record_per_page as f64)).ceil() as i32;
    let off_set = (page - 1) * record_per_page as i32;
    let pagination = create_pagination(total_pages, page).await;

    let start_record = (page - 1) * record_per_page as i32 + 1;
    let end_record = (page * record_per_page as i32).min(total_records);

    let records = match sqlx::query_as::<_, NoteResponse>(
        "SELECT * FROM notes ORDER BY id DESC LIMIT $1 OFFSET $2",
    )
    .bind(record_per_page)
    .bind(off_set)
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
            "data": records,
            "current_range": format!("Showing {} to {} of {} tasks", start_record, end_record, total_records),
            "pagination": pagination
        })),
    )
}

async fn create_pagination(pages: i32, current: i32) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    let window = 6;
    if pages < window {
        let result: Vec<String> = (1..=pages).map(|p| p.to_string()).collect();
        return result;
    }
    let buffer = 2;
    let start_window = (current - buffer).max(1);
    let end_window = (current + buffer).min(pages);
    result.push("1".to_string());
    result.push("2".to_string());
    if start_window > 3 {
        result.push("...".to_string());
    }

    for page in start_window.max(3)..=end_window {
        result.push(page.to_string());
    }

    if end_window < pages - 1 {
        result.push("...".to_string());
    }

    if end_window < pages {
        result.push(pages.to_string());
    }
    result
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
    if sqlx::query("INSERT INTO notes (summary, info_type) VALUES (?, ?)")
        .bind(&payload.summary)
        .bind(&payload.info_type)
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
            "message": "Task has been created",
            "status": 200
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
    if sqlx::query("UPDATE notes SET summary = ?, is_completed = ?, info_type = ?, updated_at = ? WHERE id = ?")
        .bind(&payload.summary)
        .bind(&payload.is_completed)
        .bind(&payload.info_type)
        .bind(chrono::Utc::now())
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
            "message": "Task has been updated!",
            "status": 200
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
            "message": "Task has been deleted",
            "status": 200
        })),
    )
}

fn resolve_todo_state(
    status: Option<String>,
    is_completed: Option<bool>,
) -> Result<(String, bool), ()> {
    if let Some(s) = status.as_deref() {
        if !matches!(s, "pending" | "in_progress" | "completed") {
            return Err(());
        }
    }
    let completed = status.as_deref() == Some("completed") || is_completed == Some(true);
    if completed {
        Ok(("completed".to_string(), true))
    } else {
        Ok((status.unwrap_or_else(|| "pending".to_string()), false))
    }
}

async fn get_professional_todos(
    State(state): State<AppState>,
    Query(query): Query<Pagination>,
    Query(range): Query<DateRange>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1).max(1);
    let total_records: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM professional_todos
         WHERE (?1 IS NULL OR date(completed_at) >= ?1)
           AND (?2 IS NULL OR date(completed_at) <= ?2)",
    )
    .bind(range.start)
    .bind(range.end)
    .fetch_one(&state.pool)
    .await
    .unwrap_or(0);
    let record_per_page: i64 = 100;
    let total_pages = ((total_records as f64) / (record_per_page as f64)).ceil() as i32;
    let off_set = (page - 1) * record_per_page as i32;
    let pagination = create_pagination(total_pages, page).await;

    let start_record = (page - 1) * record_per_page as i32 + 1;
    let end_record = (page * record_per_page as i32).min(total_records);

    let records = match sqlx::query_as::<_, ProfessionalTodoResponse>(
        "SELECT id, todo, is_completed, CAST(completed_at AS TEXT) AS completed_at, status, created_at, updated_at
         FROM professional_todos
         WHERE (?1 IS NULL OR date(completed_at) >= ?1)
           AND (?2 IS NULL OR date(completed_at) <= ?2)
         ORDER BY CASE WHEN status = 'in_progress' THEN 0 ELSE 1 END, id DESC
         LIMIT ?3 OFFSET ?4",
    )
    .bind(range.start)
    .bind(range.end)
    .bind(record_per_page)
    .bind(off_set)
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
            "data": records,
            "current_range": format!("Showing {} to {} of {} todos", start_record, end_record, total_records),
            "pagination": pagination
        })),
    )
}

async fn get_professional_todo(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    match sqlx::query_as::<_, ProfessionalTodoResponse>(
        "SELECT id, todo, is_completed, CAST(completed_at AS TEXT) AS completed_at, status, created_at, updated_at FROM professional_todos WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    {
        Ok(Some(record)) => (StatusCode::OK, Json(json!({ "data": record }))),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "message": "Todo not found"
            })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "message": "Something went wrong!"
            })),
        ),
    }
}

async fn create_professional_todo(
    ValidToken: ValidToken,
    State(state): State<AppState>,
    Json(payload): Json<ProfessionalTodoRequest>,
) -> impl IntoResponse {
    let (status, completed) = match resolve_todo_state(payload.status, payload.is_completed) {
        Ok(value) => value,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "message": "Invalid status. Use pending, in_progress or completed"
                })),
            );
        }
    };
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
    if sqlx::query(
        "INSERT INTO professional_todos (todo, is_completed, status, completed_at)
         VALUES (?, ?, ?, CASE WHEN ? = 1 THEN datetime('now') ELSE NULL END)",
    )
    .bind(&payload.todo)
    .bind(completed)
    .bind(&status)
    .bind(completed)
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
            "message": "Todo has been created",
            "status": 200
        })),
    )
}

async fn update_professional_todo(
    ValidToken: ValidToken,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<ProfessionalTodoRequest>,
) -> impl IntoResponse {
    let (status, completed) = match resolve_todo_state(payload.status, payload.is_completed) {
        Ok(value) => value,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "message": "Invalid status. Use pending, in_progress or completed"
                })),
            );
        }
    };
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
    let result = match sqlx::query(
        "UPDATE professional_todos
         SET todo = ?,
             is_completed = ?,
             status = ?,
             completed_at = CASE WHEN ? = 1 THEN COALESCE(completed_at, datetime('now')) ELSE NULL END,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = ?",
    )
    .bind(&payload.todo)
    .bind(completed)
    .bind(&status)
    .bind(completed)
    .bind(id)
    .execute(&mut *tx)
    .await
    {
        Ok(result) => result,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "message": "Something went wrong!"
                })),
            );
        }
    };
    if result.rows_affected() == 0 {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "message": "Todo not found"
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
            "message": "Todo has been updated!",
            "status": 200
        })),
    )
}

async fn delete_professional_todo(
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
    let result = match sqlx::query("DELETE FROM professional_todos WHERE id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await
    {
        Ok(result) => result,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "message": "Something went wrong!"
                })),
            );
        }
    };
    if result.rows_affected() == 0 {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "message": "Todo not found"
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
            "message": "Todo has been deleted",
            "status": 200
        })),
    )
}

async fn ensure_profile_exists(content: &str) {
    let profile_path = std::path::Path::new("./profile.json");
    let fallback_path = std::path::Path::new("/app_defaults/profile.json"); // Support for Docker local volume

    if !profile_path.exists() {
        if fallback_path.exists() {
            if let Err(e) = tokio::fs::copy(fallback_path, profile_path).await {
                eprintln!("⚠️ Failed to copy fallback profile.json: {e}");
            } else {
                println!("✅ Initialized profile.json inside mounted volume from defaults.");
            }
        } else {
            let _ = tokio::fs::write(profile_path, content).await;
            println!("✅ Created fresh profile.json inside volume.");
        }
    }
}
