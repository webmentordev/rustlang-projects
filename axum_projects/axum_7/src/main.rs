use axum::{
    Json, Router,
    extract::{Path, Query},
    http::HeaderMap,
    routing::{get, post},
};
use serde::Deserialize;

#[derive(Deserialize)]
struct Pagination {
    limit: u32,
    page: u32,
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(|| async { "hello there" }))
        .route("/users/{user_id}/posts/{post_id}", get(get_user_post))
        .route("/info", get(list_items))
        .route("/get-header", post(protected_route))
        .route("/complex/{id}", post(complex_handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!(
        "🚀 Server is running at http://{}",
        listener.local_addr().unwrap()
    );
    axum::serve(listener, app).await.unwrap();
}

async fn get_user_post(Path((user_id, post_id)): Path<(u32, u32)>) -> String {
    format!("User {} Post {}", user_id, post_id)
}

async fn list_items(Query(params): Query<Pagination>) -> String {
    format!("{} Page & Limit {}", params.page, params.limit)
}

async fn protected_route(headers: HeaderMap) -> Result<String, &'static str> {
    let auth = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or("Missing Authorization header")?;

    Ok(format!("Token: {}", auth))
}

async fn complex_handler(
    Path(id): Path<u32>,
    Query(query): Query<std::collections::HashMap<String, String>>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> String {
    format!(
        "ID: {}, Body: {:?} Query {:?} Header {:?}",
        id, body, query, headers
    )
}
