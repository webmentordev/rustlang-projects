use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};

async fn handle_connection(stream: TcpStream) {
    let ws_stream = accept_async(stream)
        .await
        .expect("Error during WebSocket handshake");
    
    println!("New WebSocket connection established");

    let (mut write, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                println!("Received: {}", text);

                let response = format!("Echo: {}", text);
                write.send(Message::Text(response)).await.ok();
            }
            Ok(Message::Binary(bin)) => {
                println!("Received binary data: {} bytes", bin.len());
                write.send(Message::Binary(bin)).await.ok();
            }
            Ok(Message::Close(_)) => {
                println!("Client disconnected");
                break;
            }
            _ => {}
        }
    }
}

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await.expect("Failed to bind");
    
    println!("WebSocket server listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream));
    }
}