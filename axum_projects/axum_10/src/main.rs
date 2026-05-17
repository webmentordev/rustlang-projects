use axum::{
    Router,
    extract::ws::{WebSocket, WebSocketUpgrade},
    routing::get,
};
use std::net::SocketAddr;

async fn ws_handler(ws: WebSocketUpgrade) -> impl axum::response::IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(msg) = socket.recv().await {
        let msg = if let Ok(msg) = msg {
            msg
        } else {
            println!("Client disconnected!");
            return;
        };
        if socket.send(msg.clone()).await.is_err() {
            println!("Client disconnected!");
            return;
        } else {
            println!("{:?}", msg.into_data());
        }
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/ws", get(ws_handler));
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
