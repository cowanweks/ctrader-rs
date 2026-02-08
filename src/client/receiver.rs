use futures_util::stream::{SplitStream, StreamExt};
use std::sync::Arc;
use tokio::{net::TcpStream, sync::Mutex};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, tungstenite::Message};

pub async fn on_message(
    incoming: Arc<Mutex<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>>,
) {
    // let incoming = incoming.clone();

    tracing::info!("Async Client Listening to Ctrader Messages");

    while let Some(msg) = incoming.lock().await.next().await {
        match msg {
            Ok(msg) => {
                match msg {
                    Message::Text(text) => {
                        tracing::info!("Text message: {}", text);
                    }
                    Message::Binary(data) => {
                        tracing::info!("Binary message: {} bytes", data.len());
                        // If it's protobuf, you'd decode it here
                    }
                    Message::Close(_) => {
                        tracing::warn!("Connection closed");
                        break;
                    }
                    _ => {
                        tracing::info!("Other message type");
                    }
                }
            }
            Err(e) => {
                tracing::error!("Error reading message: {}", e);
                continue;
            }
        }
    }
}
