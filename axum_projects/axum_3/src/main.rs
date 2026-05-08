use axum::{
    Json, Router,
    extract::Path,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Deserialize, Serialize)]
struct User {
    id: u32,
    name: String,
    email: String,
}

#[derive(Deserialize, Serialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(|| async { "Welcome to the axum ✌️" }))
        .route("/users", get(list_users))
        .route("/users", post(create_user))
        .route("/users/{id}", get(get_user));
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!(
        "🚀 Server is running at http://{}",
        &listener.local_addr().unwrap().port()
    );
    axum::serve(listener, app).await.unwrap();
}

async fn list_users() -> Json<Vec<User>> {
    let users = vec![
        User {
            id: 1,
            name: "Ahmer".to_string(),
            email: "ahmer@example.com".to_string(),
        },
        User {
            id: 2,
            name: "Junaid".to_string(),
            email: "juanid@example.com".to_string(),
        },
    ];
    Json(users)
}

async fn create_user(Json(req): Json<CreateUserRequest>) -> Json<User> {
    let user = User {
        id: 3,
        name: req.name,
        email: req.email,
    };
    Json(user)
}

async fn get_user(Path(id): Path<u32>) -> Json<User> {
    Json(User {
        id,
        name: "Kaleem".to_string(),
        email: "kaleem@example.com".to_string(),
    })
}
