use std::time::Duration;

#[derive(thiserror::Error, Debug)]
pub enum CTraderError {
    #[error("WebSocket error: {0}")]
    WebSocket(Box<tokio_tungstenite::tungstenite::Error>),

    /// Error for when a module is not found.
    #[error("Module '{0}' not found.")]
    ModuleNotFound(String),

    #[error("HTTP request error: {0}")]
    HttpRequest(String),

    #[error("Other error: {0}")]
    Other(String),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Tracing error: {0}")]
    Tracing(String),

    #[error("Failed to execute '{task}' task before the maximum allowed time of '{duration:?}'")]
    TimeoutError { task: String, duration: Duration },
}

#[derive(thiserror::Error, Debug)]
pub enum ConnectorError {}

pub type CTraderResult<T> = std::result::Result<T, CTraderError>;
pub type ConnectorResult<T> = std::result::Result<T, ConnectorError>;
