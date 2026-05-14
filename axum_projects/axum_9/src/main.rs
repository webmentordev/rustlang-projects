use axum::{Json, Router, extract::State, routing::get};
use parking_lot::Mutex;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
struct AppState {
    counter: Arc<RwLock<i32>>,
    name: String,
    data: Arc<Mutex<Vec<String>>>,
}

#[derive(Debug, Clone)]
struct Config {
    port: u16,
    debug: bool,
}

#[derive(Clone)]
struct Database {
    connection_string: String,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        counter: Arc::new(RwLock::new(0)),
        name: "MyApp".to_string(),
        data: Arc::new(Mutex::new(Vec::new())),
    };

    let app = Router::new()
        .route("/count", get(get_count))
        .route("/increment", get(increment_count))
        .route("/add-item", get(add_item))
        .route("/get-items", get(get_items))
        // .route("/handle", get(handler)) // Does not work
        .with_state(state);

    // Does not work
    // let app = Router::new()
    //     .route("/", get(handler))
    //     .with_state(Config {
    //         port: 3000,
    //         debug: true,
    //     })
    //     .with_state(Database {
    //         connection_string: "sqlite://db".to_string(),
    //     });

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    println!("🚀 Server running on http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn get_count(State(state): State<AppState>) -> String {
    let count = state.counter.read().await;
    format!("{} Count: {}", state.name, count)
}

async fn increment_count(State(state): State<AppState>) -> String {
    let mut count = state.counter.write().await;
    *count += 1;
    format!("Count: {}", count)
}

async fn add_item(State(shared): State<AppState>) -> String {
    let mut items = shared.data.lock();
    items.push("New Item".to_string());
    format!("Items: {}", items.len())
}

async fn get_items(State(shared): State<AppState>) -> Json<Vec<String>> {
    let items = shared.data.lock();
    Json(items.clone())
}

// Does not work
// async fn handler(State(config): State<Config>, State(db): State<Database>) -> String {
//     format!("Config: {:?}, DB: {}", config, db.connection_string)
// }
