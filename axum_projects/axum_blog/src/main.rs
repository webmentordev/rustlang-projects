use axum::{
    Json, Router,
    extract::{Multipart, Path, State},
    http::StatusCode,
    routing::{get, post},
};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use std::{env, net::SocketAddr};
use tokio::fs;
use tower_http::services::ServeDir;

#[derive(Deserialize, Serialize, sqlx::FromRow)]
struct Blog {
    title: String,
    slug: String,
    content: String,
    description: String,
    is_published: Option<bool>,
    created_at: Option<NaiveDateTime>,
    updated_at: Option<NaiveDateTime>,
    small_thumbnail: Option<String>,
    large_thumbnail: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct Response {
    message: String,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().unwrap();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not found!");
    let pool = PgPoolOptions::new()
        .max_connections(100) // tune based on your postgres server's max_connections
        .min_connections(10)
        .connect(&database_url)
        .await
        .unwrap();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS blogs (
        id SERIAL PRIMARY KEY,
        title VARCHAR(255) NOT NULL,
        slug VARCHAR(255) NOT NULL,
        content TEXT NOT NULL,
        description TEXT NOT NULL,
        small_thumbnail TEXT,
        large_thumbnail TEXT,
        is_published BOOLEAN DEFAULT TRUE,
        created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    )",
    )
    .execute(&pool)
    .await
    .unwrap();

    let blog_routes = Router::new()
        .route("/", get(get_blogs))
        .route("/{slug}", get(get_blog));
    let admin_routes = Router::new().route("/create", post(write_blog));
    let api_route = Router::new().nest("/posts", blog_routes);
    let app = Router::new()
        .nest("/api", api_route)
        .nest("/admin", admin_routes)
        .nest_service("/uploads", ServeDir::new("./uploads"))
        .with_state(pool);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("🚀 Server running at http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn get_blogs(State(pool): State<sqlx::PgPool>) -> Json<Vec<Blog>> {
    let results = sqlx::query_as::<_, Blog>(
        "SELECT title, slug, content, description, is_published, created_at, updated_at, small_thumbnail, large_thumbnail FROM blogs",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    Json(results)
}

async fn get_blog(
    Path(slug): Path<String>,
    State(pool): State<sqlx::PgPool>,
) -> Result<Json<Blog>, StatusCode> {
    let result = sqlx::query_as::<_, Blog>(
        "SELECT title, slug, content, description, is_published, created_at, updated_at, small_thumbnail, large_thumbnail FROM blogs WHERE slug = $1",
    )
    .bind(&slug)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match result {
        Some(blog) => Ok(Json(blog)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn write_blog(
    State(pool): State<sqlx::PgPool>,
    mut multipart: Multipart,
) -> Result<Json<Response>, StatusCode> {
    let mut title = String::new();
    let mut slug = String::new();
    let mut content = String::new();
    let mut description = String::new();
    let mut small_thumbnail: Option<String> = None;
    let mut large_thumbnail: Option<String> = None;

    fs::create_dir_all("./uploads")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "title" => title = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?,
            "slug" => slug = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?,
            "content" => content = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?,
            "description" => {
                description = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?
            }
            "small_thumbnail" | "large_thumbnail" => {
                let filename = field.file_name().unwrap_or("file").to_string();
                let ext = filename.rsplit('.').next().unwrap_or("jpg");
                let stem = filename.rsplitn(2, '.').last().unwrap_or(&filename);
                let final_name = format!("{}.{}", generate_slug(stem), ext);
                let path = format!("./uploads/{}", final_name);
                let url = format!("/uploads/{}", final_name);

                let bytes = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                fs::write(&path, bytes)
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                if name == "small_thumbnail" {
                    small_thumbnail = Some(url);
                } else {
                    large_thumbnail = Some(url);
                }
            }
            _ => {}
        }
    }

    if small_thumbnail.is_none() || large_thumbnail.is_none() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let result = sqlx::query(
        "INSERT INTO blogs (title, slug, content, description, small_thumbnail, large_thumbnail)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(&title)
    .bind(generate_slug(&slug))
    .bind(&content)
    .bind(&description)
    .bind(&small_thumbnail)
    .bind(&large_thumbnail)
    .execute(&pool)
    .await;

    match result {
        Ok(_) => Ok(Json(Response {
            message: "Blog has been posted".to_string(),
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

fn generate_slug(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
