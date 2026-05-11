use axum::{
    Json, Router,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use serde_json::json;
use std::net::SocketAddr;

type AppResult<T> = Result<T, AppError>;

#[derive(Debug)]
enum AppError {
    NotFound,
    InvalidInput(String),
    InternalError,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "Resource not found".into()),
            AppError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::InternalError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".into(),
            ),
        };
        let body = Json(json!({
            "error": message
        }));
        (status, body).into_response()
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/item/{id}", get(get_item))
        .route("/validate", get(validate_input));
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("Server running on http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn get_item(axum::extract::Path(id): axum::extract::Path<u32>) -> Result<String, AppError> {
    if id == 0 {
        Err(AppError::NotFound)
    } else {
        Ok(format!("Item {}", id))
    }
}

async fn validate_input(
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> AppResult<String> {
    let name = params
        .get("name")
        .ok_or_else(|| AppError::InvalidInput("name parameter required".to_string()))?;

    if name.len() < 3 {
        return Err(AppError::InvalidInput(
            "name must be at least 3 characters".to_string(),
        ));
    }
    Ok(format!("Valid name: {}", name))
}
