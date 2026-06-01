use std::net::SocketAddr;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

#[derive(Serialize, Deserialize)]
struct Response {
    total_records: i64,
    total_pages: i32,
    current_page: i32,
    current_records: i32,
    fields: Vec<Fields>,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
struct Fields {
    id: i32,
    title: String,
    created_at: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize)]
struct RequestData {
    title: String,
}

#[derive(Clone)]
struct AppState {
    pool: Pool<Postgres>,
}

#[tokio::main]
async fn main() {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgresql://axum:mypasswor1235123@localhost:5444/axum_db")
        .await
        .unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS items (
        id SERIAL PRIMARY KEY,
        title VARCHAR(255) NOT NULL,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    )",
    )
    .execute(&pool)
    .await
    .unwrap();

    let app = Router::new()
        // I did not use Query params here as this is just a test project. You can update it as you like.
        .route("/{total}/{page}", get(get_data))
        .route("/", post(save_data))
        .with_state(AppState { pool });
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("🚀 Server is running at: http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn get_data(
    State(state): State<AppState>,
    Path((record_per_page, current_page)): Path<(i32, i32)>,
) -> impl IntoResponse {
    let total_records = match sqlx::query_scalar::<_, i64>("SELECT COUNT(*) from items")
        .fetch_one(&state.pool)
        .await
    {
        Ok(count) => count,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "message": "Something went wrong!"
                })),
            );
        }
    };

    let total_pages = ((total_records as f64) / (record_per_page as f64)).ceil() as i32;
    let off_set = (current_page - 1) * record_per_page;

    match sqlx::query_as::<_, Fields>(
        "SELECT id, title, created_at FROM items ORDER BY created_at DESC LIMIT $1 OFFSET $2",
    )
    .bind(record_per_page)
    .bind(off_set)
    .fetch_all(&state.pool)
    .await
    {
        Ok(fields) => (
            StatusCode::OK,
            Json(json!({
                "message": "Data fetched!",
                "fields": Response{
                    total_records,
                    total_pages,
                    current_page,
                    current_records: fields.len() as i32,
                    fields
                }
            })),
        ),
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "message": "Data not found!"
            })),
        ),
    }
}

async fn save_data(
    State(state): State<AppState>,
    Json(payload): Json<RequestData>,
) -> impl IntoResponse {
    match sqlx::query("INSERT into items (title) VALUES($1)")
        .bind(&payload.title)
        .execute(&state.pool)
        .await
    {
        Ok(records) if records.rows_affected() > 0 => (
            StatusCode::OK,
            Json(json!({
                "message": "Record saved!",
            })),
        ),
        Ok(_) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({
                "message": "Could not save the content!"
            })),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "message": "Internal server error!"
            })),
        ),
    }
}
