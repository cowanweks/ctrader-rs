use futures_util::stream::SplitSink;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};

/// CTrader Auth details
/// * ctrader_client_id - CTrader Application Client ID
/// * ctrader_client_secret - CTrader Application Secret
/// * ctrader_redirect_url - Ctrader Application Redirect URL
/// * ctrader_refresh_token - Ctrader Application Refresh Token
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Auth {
    pub ctrader_access_token: String,
    pub ctrader_client_id: String,
    pub ctrader_client_secret: String,
    pub ctrader_redirect_url: String,
    pub ctrader_refresh_token: String,
}

/// The CTrader client instance
/// * auth - The Authorization information to use.
/// * write_stream - The websocket stream to use to send to the websocket server.
/// * read_stream - The websocket stream to use to receive messages from the websocket server.
//
//
//

//
//
#[derive(Debug, Clone)]
pub struct CTraderClient {
    pub auth: Auth,

    pub ws_write: Arc<
        Mutex<
            SplitSink<
                WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
                Message,
            >,
        >,
    >,
}

/// The representation of the response from the get token request.
#[derive(Serialize, Deserialize)]
pub struct TokenResponse;
