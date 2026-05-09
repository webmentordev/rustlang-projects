use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::IntoResponse,
    routing::get,
};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(hello))
        .route(
            "/protected",
            get(protected).layer(middleware::from_fn(auth_middleware)), // Applies on specific route
        )
        .layer(middleware::from_fn(logging)); // Applies on all

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("🚀 Server is running at: http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn hello() -> String {
    "Hello".to_string()
}

async fn logging(req: Request<Body>, next: Next) -> impl IntoResponse {
    let method = req.method().clone();
    let uri = req.uri().clone();
    println!("Request: {} {}", method, uri);

    let response = next.run(req).await;
    println!("Response status: {}", response.status());
    response
}

async fn protected() -> String {
    "Protected!".to_string()
}

async fn auth_middleware(req: Request<Body>, next: Next) -> Result<impl IntoResponse, StatusCode> {
    let auth_header = req.headers().get("Authorization");
    match auth_header {
        Some(header) if header.to_str().unwrap_or("").starts_with("Bearer ") => {
            Ok(next.run(req).await)
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}
