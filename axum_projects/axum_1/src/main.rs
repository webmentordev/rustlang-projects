use std::net::SocketAddr;

use axum::{Router, routing::get};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(|| async { "Welcome to the axum" }))
        .route("/ping", get(|| async { "Pong!" }))
        .route("/hello", get(hello_world));
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    println!("Server is running at http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn hello_world() -> String {
    "Hello world!".to_string()
}
