use axum::{Router, extract::Path, routing::get};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let users_routes = Router::new()
        .route("/", get(list_users).post(create_user))
        .route("/{id}", get(get_user).put(update_user).delete(delete_user));
    let posts_routes = Router::new()
        .route("/", get(list_posts).post(create_post))
        .route("/{id}", get(get_post).delete(delete_post));
    let api_routes = Router::new()
        .nest("/users", users_routes)
        .nest("/posts", posts_routes);
    let app = Router::new()
        .nest("/api", api_routes)
        .route("/health", get(health));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("🚀 Server running on http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> String {
    "OK".to_string()
}

async fn list_users() -> String {
    "Users".to_string()
}

async fn create_user() -> String {
    "Created".to_string()
}

async fn get_user(Path(id): Path<u32>) -> String {
    format!("User {}", id)
}

async fn update_user(Path(id): Path<u32>) -> String {
    format!("Updated {}", id)
}

async fn delete_user(Path(id): Path<u32>) -> String {
    format!("Deleted {}", id)
}

async fn list_posts() -> String {
    "Posts".to_string()
}

async fn create_post() -> String {
    "Post Created".to_string()
}

async fn get_post(Path(id): Path<u32>) -> String {
    format!("Post {}", id)
}

async fn delete_post(Path(id): Path<u32>) -> String {
    format!("Post {} Deleted", id)
}
