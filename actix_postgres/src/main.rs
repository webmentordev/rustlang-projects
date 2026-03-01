mod db;
mod routes;

use actix_web::{App, HttpServer, Responder, web};
use dotenv::dotenv;
use routes::users::*;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let pool = db::create_pool().await;
    let port = 8050;
    println!("🚀 Listening at http://127.0.0.1:{}", port);

    // This is optional
    // It will auto run the migrations on each run.
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let pool = web::Data::new(pool);

    HttpServer::new(move || {
        App::new()
            .app_data(pool.clone())
            .route("/", web::get().to(index))
            .route("/users", web::get().to(get_users))
            .route("/users", web::post().to(create_user))
            .route("/users/{id}", web::get().to(get_user))
            .route("/users/{id}", web::delete().to(delete_user))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}

async fn index() -> impl Responder {
    "Welcome to the Actix Web and Postgres project"
}
