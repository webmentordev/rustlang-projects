use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};

#[tokio::main]
async fn main() {
    let url = "ws://127.0.0.1:8080";
    
    println!("Connecting to {}", url);
    
    let (ws_stream, _) = connect_async(url)
        .await
        .expect("Failed to connect");
    
    println!("Connected to WebSocket server");

    let (mut write, mut read) = ws_stream.split();

    let messages = vec![
        "Hello, WebSocket!",
        "This is a test message",
        "Goodbye!",
    ];

    for msg in messages {
        println!("Sending: {}", msg);
        write.send(Message::Text(msg.to_string())).await.ok();
        
        if let Some(response) = read.next().await {
            match response {
                Ok(Message::Text(text)) => {
                    println!("Received: {}", text);
                }
                _ => {}
            }
        }
        
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    write.send(Message::Close(None)).await.ok();
    println!("Connection closed");
}