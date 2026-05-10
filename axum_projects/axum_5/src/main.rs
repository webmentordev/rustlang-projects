use axum::{
    Json, Router,
    extract::Path,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePoolOptions;
use std::net::SocketAddr;

#[derive(Serialize, Deserialize, sqlx::FromRow)]
struct User {
    id: i32,
    name: String,
    email: String,
}

#[derive(Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

#[tokio::main]
async fn main() {
    let database_url = "sqlite::memory:";
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .unwrap();

    sqlx::query(
        "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL, email TEXT NOT NULL)",
    )
    .execute(&pool)
    .await
    .unwrap();

    let app = Router::new()
        .route("/users", get(list_users))
        .route("/users", post(create_user))
        .route("/users/{id}", get(get_user))
        .with_state(pool);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("Server is running at http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn list_users(
    axum::extract::State(pool): axum::extract::State<sqlx::SqlitePool>,
) -> Json<Vec<User>> {
    let users = sqlx::query_as::<_, User>("SELECT id, name, email FROM users")
        .fetch_all(&pool)
        .await
        .unwrap_or_default();
    Json(users)
}

async fn create_user(
    axum::extract::State(pool): axum::extract::State<sqlx::SqlitePool>,
    Json(req): Json<CreateUserRequest>,
) -> Json<User> {
    let result = sqlx::query("INSERT INTO users (name, email) VALUES (?, ?)")
        .bind(&req.name)
        .bind(&req.email)
        .execute(&pool)
        .await
        .unwrap();
    let user = User {
        id: result.last_insert_rowid() as i32,
        name: req.name,
        email: req.email,
    };
    Json(user)
}

async fn get_user(
    axum::extract::State(pool): axum::extract::State<sqlx::SqlitePool>,
    Path(id): Path<i32>,
) -> Json<Option<User>> {
    let user = sqlx::query_as::<_, User>("SELECT id, name, email FROM users WHERE id = ?")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .unwrap();
    Json(user)
}
