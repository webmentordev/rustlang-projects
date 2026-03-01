use sqlx::{PgPool, postgres::PgPoolOptions};
use std::env;

pub async fn create_pool() -> PgPool {
    let databse_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgPoolOptions::new()
        .max_connections(50)
        .connect(&databse_url)
        .await
        .expect("Failt to connect to Postgres")
}
