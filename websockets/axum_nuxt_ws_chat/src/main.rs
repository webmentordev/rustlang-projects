use axum::{
    Router,
    extract::{
        Path, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
    routing::get,
};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::broadcast;

#[derive(Deserialize, Serialize)]
struct Room {
    message: String,
}

#[derive(Clone)]
struct AppState {
    rooms: Arc<DashMap<String, broadcast::Sender<String>>>,
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/ws/{room}", get(handle_ws))
        .with_state(AppState {
            rooms: Arc::new(DashMap::new()),
        });

    let port = SocketAddr::from(([0, 0, 0, 0], 3099));
    let listener = tokio::net::TcpListener::bind(port).await.unwrap();
    println!("Server running at: http://127.0.0.1:{}", port.port());
    axum::serve(listener, app).await.unwrap();
}

async fn handle_ws(
    ws: WebSocketUpgrade,
    Path(room): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // println!("Connection Received ROOM: {}", room);
    ws.on_upgrade(move |socket| handle_socket(socket, room, state))
}

async fn handle_socket(mut socket: WebSocket, room: String, state: AppState) {
    let tx = state
        .rooms
        .entry(room.clone())
        .or_insert_with(|| broadcast::channel(100).0)
        .clone();
    let mut rx = tx.subscribe();

    loop {
        tokio::select! {
            Ok(msg) = rx.recv() => {
                if socket.send(Message::Text(msg.into())).await.is_err() {
                    break;
                }
            }
            incoming = socket.recv() => {
                match incoming {
                    Some(Ok(Message::Text(text))) => {
                        println!("Received: {}", text);
                        let request: Room = match serde_json::from_str(text.as_str()) {
                            Ok(r) => r,
                            // Err(e) => { eprintln!("bad message: {e}"); continue; }
                            Err(_) => { continue; }
                        };
                        // TODO: Auth user's name
                        let reply = format!("Your message received: {}", request.message);
                        let _ = tx.send(reply);
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        // println!("Client disconnected");
                        break;
                    }
                    Some(Err(_)) => break,
                    _ => {}
                }
            }
        }
    }
}
