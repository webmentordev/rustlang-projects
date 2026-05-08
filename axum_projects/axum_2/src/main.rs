use std::net::SocketAddr;

use axum::{
    Router,
    routing::{delete, get, post, put},
};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(home))
        .route("/users", get(list_users))
        .route("/users", post(create_user))
        .route("/users/{id}", get(get_user))
        .route("/users/{id}", put(update_user))
        .route("/users/{id}", delete(delete_user));
    // {id} is new syntax, :id was in 0.7 (as I read in the blog)
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("Server running at http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn home() -> String {
    "Home page".to_string()
}

async fn list_users() -> String {
    "User list".to_string()
}

async fn create_user() -> String {
    "User created".to_string()
}

async fn get_user(axum::extract::Path(id): axum::extract::Path<u32>) -> String {
    format!("User {}", id)
}

async fn update_user(axum::extract::Path(id): axum::extract::Path<u32>) -> String {
    format!("User {} updated", id)
}

async fn delete_user(axum::extract::Path(id): axum::extract::Path<u32>) -> String {
    format!("User {} deleted", id)
}
