use crate::{error::ConnectorResult, traits::AppState};
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

pub type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub trait Connector<S: AppState>: Send + Sync {
    /// Connect to the WebSocket server and return the stream
    async fn connect(&self, state: S) -> ConnectorResult<WsStream>;

    /// Disconnect from the WebSocket server
    async fn disconnect(&self) -> ConnectorResult<()>;

    /// Reconnect to the WebSocket server with automatic retry logic and return the stream
    async fn reconnect(&self, state: S) -> ConnectorResult<WsStream> {
        self.disconnect().await?;

        // Retry logic can be implemented here if needed
        self.connect(state).await
    }
}
