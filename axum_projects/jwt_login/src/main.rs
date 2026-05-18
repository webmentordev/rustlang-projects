use axum::{Json, Router, extract::FromRequestParts, http::request::Parts, routing::post};
use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Deserialize, Serialize)]
struct Claims {
    sub: String,
    exp: i64,
}

#[derive(Deserialize)]
struct Login {
    email: String,
    password: String,
}

struct AuthUser(String);

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = String;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .ok_or("Missing token")?;
        let secret = env::var("JWT_SECRET").unwrap();
        let key = DecodingKey::from_secret(secret.as_bytes());
        let claims =
            decode::<Claims>(token, &key, &Validation::default()).map_err(|_| "Invalid token")?;
        Ok(AuthUser(claims.claims.sub))
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let app = Router::new()
        .route("/login", post(login))
        .route("/protected", post(protected));
    axum::serve(
        tokio::net::TcpListener::bind("127.0.0.1:3000")
            .await
            .unwrap(),
        app,
    )
    .await
    .unwrap();
}

async fn login(Json(Login { email, password }): Json<Login>) -> String {
    if email == "admin@gmail.com" && password == "password" {
        let secret = env::var("JWT_SECRET").unwrap();
        let key = EncodingKey::from_secret(secret.as_bytes());
        let claims = Claims {
            sub: email,
            exp: Utc::now().timestamp() * 3600,
        };
        encode(&Header::default(), &claims, &key).unwrap()
    } else {
        String::new()
    }
}

async fn protected(AuthUser(user): AuthUser) -> String {
    format!("Hello {}", user)
}
