use axum::{
    Router,
    extract::{
        Path,
        ws::{Message as WsMessage, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
    routing::get,
};
use bytes::Bytes;
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;

#[derive(Clone, Debug)]
enum AudioMessage {
    Audio { data: Bytes, from: String },
}

struct AppState {
    rooms: std::sync::Mutex<std::collections::HashMap<String, broadcast::Sender<AudioMessage>>>,
}

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState {
        rooms: std::sync::Mutex::new(std::collections::HashMap::new()),
    });

    let app = Router::new()
        .route("/ws/{room_id}", get(ws_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3939")
        .await
        .unwrap();

    println!("Server running on http://127.0.0.1:3939");

    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(
    Path(room_id): Path<String>,
    ws: WebSocketUpgrade,
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, room_id, state))
}

async fn handle_socket(socket: WebSocket, room_id: String, state: Arc<AppState>) {
    let user_id = Uuid::new_v4().to_string();
    let (mut sender, mut receiver) = socket.split();

    let tx = {
        let mut rooms = state.rooms.lock().unwrap();
        rooms
            .entry(room_id.clone())
            .or_insert_with(|| broadcast::channel::<AudioMessage>(100).0)
            .clone()
    };

    let mut rx = tx.subscribe();

    let user_id_recv = user_id.clone();
    let user_id_send = user_id.clone();
    let tx_clone = tx.clone();

    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let WsMessage::Binary(data) = msg {
                let audio_msg = AudioMessage::Audio {
                    data,
                    from: user_id_recv.clone(),
                };
                let _ = tx_clone.send(audio_msg);
            }
        }
    });

    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if let AudioMessage::Audio { data, from } = &msg {
                if from != &user_id_send {
                    if sender.send(WsMessage::Binary(data.clone())).await.is_err() {
                        break;
                    }
                }
            }
        }
    });

    tokio::select! {
        _ = recv_task => {},
        _ = send_task => {},
    }
}
