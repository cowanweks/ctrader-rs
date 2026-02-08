use futures_util::{SinkExt, stream::SplitSink};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, tungstenite::Message};

pub async fn send_heartbeat(
    outgoing: Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>,
) {
    loop {
        let msg = Message::Text("heartbeat".into());

        outgoing
            .lock()
            .await
            .send(msg)
            .await
            .expect("unable to send heartbeat");

        tokio::time::sleep(Duration::from_secs(30)).await;
    }
}

#[cfg(test)]
mod tests {
    use futures_util::StreamExt;
    use tokio_tungstenite::connect_async;

    use super::*;

    #[tokio::test]
    async fn test_send_heartbeat() -> anyhow::Result<()> {
        let url = "wss://echo.websockets.events";

        let (ws_stream, _) = connect_async(url)
            .await
            .expect("error connecting to socket");

        let (outgoing, incoming) = ws_stream.split();

        let incoming = Arc::new(Mutex::new(incoming));
        let outgoing = Arc::new(Mutex::new(outgoing));

        let receiver_handle = tokio::spawn(async move {
            loop {
                while let Some(message) = incoming.lock().await.next().await {
                    match message {
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
                        Err(err) => {
                            tracing::error!("Error reading message: {}", err);
                            continue;
                        }
                    }
                }
            }
        });

        let sender_handle = tokio::spawn(async {});

        let heartbeat_handle = tokio::spawn(send_heartbeat(outgoing));

        let _ = tokio::join!(receiver_handle, sender_handle, heartbeat_handle);

        Ok(())
    }
}
