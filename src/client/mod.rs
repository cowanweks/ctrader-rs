mod auth;
mod connector;
mod endpoint;
mod middleware;
mod receiver;
mod sender;

pub mod traits;

use crate::client::{receiver::on_message, sender::send_heartbeat};
use crate::openapi::{
    ProtoMessage, ProtoOaAccountAuthReq, ProtoOaAccountLogoutReq, ProtoOaApplicationAuthReq,
    ProtoOaAssetClassListReq, ProtoOaAssetListReq, ProtoOaCancelOrderReq, ProtoOaClosePositionReq,
    ProtoOaDealOffsetListReq, ProtoOaGetAccountListByAccessTokenReq,
    ProtoOaGetPositionUnrealizedPnLReq, ProtoOaGetTickDataReq, ProtoOaGetTrendbarsReq,
    ProtoOaNewOrderReq, ProtoOaOrderDetailsReq, ProtoOaOrderListByPositionIdReq, ProtoOaQuoteType,
    ProtoOaReconcileReq, ProtoOaRefreshTokenReq, ProtoOaSubscribeSpotsReq,
    ProtoOaSymbolCategoryListReq, ProtoOaSymbolsListReq, ProtoOaTraderReq,
    ProtoOaUnsubscribeSpotsReq,
};
use crate::openapi::{ProtoOaOrderType, ProtoOaTradeSide};
use endpoint::Endpoints;
use prost::Message;

use crate::types::{Auth, CTraderClient};

use futures_util::{SinkExt, StreamExt};

use std::time::Duration;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};

#[allow(dead_code)]
impl CTraderClient {
    /// Create a new CTrader OpenAPi client instance
    /// * is_demo - Specify whether to use Demo/Live host
    /// * app_client_id - CTrader Application Client ID
    /// * app_client_secret - CTrader Application Secret
    /// * app_redirect_url - Ctrader Application Redirect URL
    /// * refresh_token - CTrader Refresh Token
    pub async fn new(
        is_demo: bool,
        app_client_id: String,
        app_access_token: String,
        app_client_secret: String,
        app_redirect_url: String,
        refresh_token: String,
    ) -> Result<(Self, JoinHandle<()>, JoinHandle<()>), anyhow::Error> {
        let host = if is_demo {
            Endpoints::DEMO_HOST_WS_URI
        } else {
            Endpoints::LIVE_HOST_URI
        };

        let url = format!("{}:{}", host, Endpoints::PROTOBUF_PORT);

        tracing::info!(
            "Connecting Async Client to CTrader Endpoint {} OpenAPI",
            url
        );

        // Try connection multiple attempts
        let web_socket_stream = loop {
            let web_socket_stream = if let Ok((ws_stream, _)) = connect_async(url.as_str()).await {
                Some(ws_stream)
            } else {
                tracing::warn!("Error connecting to {} retrying in 5s ", url);
                None
            };

            if web_socket_stream.is_some() {
                tracing::info!("Successfully Connected Async Client to CTrader OpenAPI");
                break web_socket_stream.unwrap();
            }

            tokio::time::sleep(Duration::from_secs(5)).await;
        };

        let auth = Auth::new(
            app_client_id,
            app_access_token,
            app_client_secret,
            app_redirect_url,
            refresh_token,
        );

        let (ws_write, ws_read) = web_socket_stream.split();
        let incoming = Arc::new(Mutex::new(ws_read));

        let outgoing = Arc::new(Mutex::new(ws_write));

        let message_handle = tokio::spawn(on_message(incoming));
        let heartbeat_handle = tokio::spawn(send_heartbeat(outgoing.clone()));

        Ok((
            Self {
                auth,
                ws_write: outgoing.clone(),
            },
            message_handle,
            heartbeat_handle,
        ))
    }

    /// Send heartbeat to CTrader API to ensure the connection is a live
    async fn send_heartbeat(&self) {}

    /// Send a new LIMIT order request
    pub async fn send_new_limit_order(
        &self,
        account_id: i64,
        symbol_id: i64,
        trade_side: ProtoOaTradeSide,
        volume: i64,
        price: f64,
    ) -> Result<(), anyhow::Error> {
        let order_type = ProtoOaOrderType::Limit;

        self.send_new_order_request(
            account_id,
            symbol_id,
            order_type,
            trade_side,
            volume,
            Some(price),
        )
        .await?;
        Ok(())
    }

    /// Send a new MARKET order request
    pub async fn send_new_market_order(
        &self,
        account_id: i64,
        symbol_id: i64,
        trade_side: ProtoOaTradeSide,
        volume: i64,
    ) -> Result<(), anyhow::Error> {
        let order_type = ProtoOaOrderType::Market;

        self.send_new_order_request(account_id, symbol_id, order_type, trade_side, volume, None)
            .await?;
        Ok(())
    }

    /// Send a new STOP order request
    pub async fn send_new_stop_order(
        &self,
        account_id: i64,
        symbol_id: i64,
        trade_side: ProtoOaTradeSide,
        volume: i64,
        price: f64,
    ) -> Result<(), anyhow::Error> {
        let order_type = ProtoOaOrderType::Stop;

        self.send_new_order_request(
            account_id,
            symbol_id,
            order_type,
            trade_side,
            volume,
            Some(price),
        )
        .await?;
        Ok(())
    }

    pub async fn send_refresh_token_request(&self) -> Result<(), anyhow::Error> {
        tracing::info!("Refreshing Application Token");

        let req = ProtoOaRefreshTokenReq {
            refresh_token: self.auth.ctrader_refresh_token.to_string(),
            payload_type: Some(2173),
        };

        // Serialize to protobuf bytes
        let mut buf = Vec::new();
        req.encode(&mut buf)?;

        // Create message with payload type prefix
        let msg_id: u16 = 2173; // Application Auth Request ID
        let mut message = Vec::with_capacity(2 + buf.len());
        message.extend_from_slice(&msg_id.to_le_bytes()); // Message ID
        message.extend_from_slice(&buf); // Protobuf payload

        // Send as binary message
        let ws_msg = WsMessage::Binary(message.into());

        let _ = &mut self.ws_write.lock().await.send(ws_msg).await?;

        Ok(())
    }

    /// Authenticate the client to the CTrader APi
    pub async fn send_application_auth_request(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Authenticating Async Client to CTrader OpenAPI");

        let req = ProtoOaApplicationAuthReq {
            client_id: self.auth.ctrader_client_id.to_string(),
            client_secret: self.auth.ctrader_client_secret.to_string(),
            payload_type: Some(2100),
        };

        // Serialize to protobuf bytes
        let mut buf = Vec::new();
        req.encode(&mut buf)?;

        // Create message with payload type prefix
        let msg_id: u16 = 2100; // Application Auth Request ID
        let mut message = Vec::with_capacity(2 + buf.len());
        message.extend_from_slice(&msg_id.to_le_bytes()); // Message ID
        message.extend_from_slice(&buf); // Protobuf payload

        // Send as binary message
        let ws_msg = WsMessage::Binary(message.into());

        let _ = &mut self.ws_write.lock().await.send(ws_msg).await?;

        Ok(())
    }

    pub async fn send_set_account_request(&mut self, account_id: i64) -> Result<(), anyhow::Error> {
        tracing::info!("Setting Active account to {}", account_id);

        let req = ProtoOaAccountAuthReq {
            ctid_trader_account_id: account_id,
            access_token: self.auth.ctrader_access_token.to_string(),
            payload_type: Some(2102),
        };

        // Serialize to protobuf bytes
        let mut buf = Vec::new();
        req.encode(&mut buf)?;

        // Create message with payload type prefix
        let msg_id: u16 = 2102; // Application Auth Request ID
        let mut message = Vec::with_capacity(2 + buf.len());
        message.extend_from_slice(&msg_id.to_le_bytes()); // Message ID
        message.extend_from_slice(&buf); // Protobuf payload

        // Send as binary message
        let ws_msg = WsMessage::Binary(message.into());

        let _ = &mut self.ws_write.lock().await.send(ws_msg).await?;

        Ok(())
    }

    pub async fn send_get_account_list_by_access_token_request(&self) -> Result<(), anyhow::Error> {
        let req = ProtoOaGetAccountListByAccessTokenReq {
            access_token: self.auth.ctrader_access_token.to_string(),
            payload_type: Some(2149),
        };

        // Serialize to protobuf bytes
        let mut buf = Vec::new();
        req.encode(&mut buf)?;

        // Create message with payload type prefix
        let msg_id: u16 = 2149; // Application Auth Request ID
        let mut message = Vec::with_capacity(2 + buf.len());
        message.extend_from_slice(&msg_id.to_le_bytes()); // Message ID
        message.extend_from_slice(&buf); // Protobuf payload

        // Send as binary message
        let ws_msg = WsMessage::Binary(message.into());

        let _ = &mut self.ws_write.lock().await.send(ws_msg).await?;

        Ok(())
    }

    pub async fn send_account_logout_request(&self, account_id: i64) -> Result<(), anyhow::Error> {
        let req = ProtoOaAccountLogoutReq {
            ctid_trader_account_id: account_id,
            payload_type: Some(2162),
        };

        // Serialize to protobuf bytes
        let mut buf = Vec::new();
        req.encode(&mut buf)?;

        // Create message with payload type prefix
        let msg_id: u16 = 2162; // Application Auth Request ID
        let mut message = Vec::with_capacity(2 + buf.len());
        message.extend_from_slice(&msg_id.to_le_bytes()); // Message ID
        message.extend_from_slice(&buf); // Protobuf payload

        // Send as binary message
        let ws_msg = WsMessage::Binary(message.into());

        let _ = &mut self.ws_write.lock().await.send(ws_msg).await?;

        Ok(())
    }

    pub async fn send_asset_list_request(&self, account_id: i64) -> Result<(), anyhow::Error> {
        let req = ProtoOaAssetListReq {
            ctid_trader_account_id: account_id,
            payload_type: Some(2112),
        };

        // Serialize to protobuf bytes
        let mut buf = Vec::new();
        req.encode(&mut buf)?;

        // Create message with payload type prefix
        let msg_id: u16 = 2112; // Application Auth Request ID
        let mut message = Vec::with_capacity(2 + buf.len());
        message.extend_from_slice(&msg_id.to_le_bytes()); // Message ID
        message.extend_from_slice(&buf); // Protobuf payload

        // Send as binary message
        let ws_msg = WsMessage::Binary(message.into());

        let _ = &mut self.ws_write.lock().await.send(ws_msg).await?;

        Ok(())
    }

    pub async fn send_asset_class_list_request(
        &self,
        account_id: i64,
    ) -> Result<(), anyhow::Error> {
        let req = ProtoOaAssetClassListReq {
            ctid_trader_account_id: account_id,
            payload_type: Some(2153),
        };

        // Serialize to protobuf bytes
        let mut buf = Vec::new();
        req.encode(&mut buf)?;

        // Create message with payload type prefix
        let msg_id: u16 = 2153; // Application Auth Request ID
        let mut message = Vec::with_capacity(2 + buf.len());
        message.extend_from_slice(&msg_id.to_le_bytes()); // Message ID
        message.extend_from_slice(&buf); // Protobuf payload

        // Send as binary message
        let ws_msg = WsMessage::Binary(message.into());

        let _ = &mut self.ws_write.lock().await.send(ws_msg).await?;

        Ok(())
    }

    pub async fn send_symbol_category_list_request(
        &self,
        account_id: i64,
    ) -> Result<(), anyhow::Error> {
        let req = ProtoOaSymbolCategoryListReq {
            ctid_trader_account_id: account_id,
            payload_type: Some(2160),
        };

        // Serialize to protobuf bytes
        let mut buf = Vec::new();
        req.encode(&mut buf)?;

        // Create message with payload type prefix
        let msg_id: u16 = 2160; // Application Auth Request ID
        let mut message = Vec::with_capacity(2 + buf.len());
        message.extend_from_slice(&msg_id.to_le_bytes()); // Message ID
        message.extend_from_slice(&buf); // Protobuf payload

        // Send as binary message
        let ws_msg = WsMessage::Binary(message.into());

        let _ = &mut self.ws_write.lock().await.send(ws_msg).await?;

        Ok(())
    }

    pub async fn send_symbols_list_request(
        &self,
        account_id: i64,
        include_archived_symbols: bool,
    ) -> Result<(), anyhow::Error> {
        let req = ProtoOaSymbolsListReq {
            ctid_trader_account_id: account_id,
            include_archived_symbols: Some(include_archived_symbols),
            payload_type: Some(2114),
        };

        // Serialize to protobuf bytes
        let mut buf = Vec::new();
        req.encode(&mut buf)?;

        // Create message with payload type prefix
        let msg_id: u16 = 2114; // Application Auth Request ID
        let mut message = Vec::with_capacity(2 + buf.len());
        message.extend_from_slice(&msg_id.to_le_bytes()); // Message ID
        message.extend_from_slice(&buf); // Protobuf payload

        // Send as binary message
        let ws_msg = WsMessage::Binary(message.into());

        let _ = &mut self.ws_write.lock().await.send(ws_msg).await?;

        Ok(())
    }

    pub async fn send_trader_request(&self, account_id: i64) -> Result<(), anyhow::Error> {
        let req = ProtoOaTraderReq {
            ctid_trader_account_id: account_id,
            payload_type: Some(2121),
        };

        // Serialize to protobuf bytes
        let mut buf = Vec::new();
        req.encode(&mut buf)?;

        // Create message with payload type prefix
        let msg_id: u16 = 2121; // Application Auth Request ID
        let mut message = Vec::with_capacity(2 + buf.len());
        message.extend_from_slice(&msg_id.to_le_bytes()); // Message ID
        message.extend_from_slice(&buf); // Protobuf payload

        // Send as binary message
        let ws_msg = WsMessage::Binary(message.into());

        let _ = &mut self.ws_write.lock().await.send(ws_msg).await?;

        Ok(())
    }

    pub async fn send_unsubscribe_spots_request(
        &self,
        account_id: i64,
        symbol_id: Vec<i64>,
    ) -> Result<(), anyhow::Error> {
        let req = ProtoOaUnsubscribeSpotsReq {
            ctid_trader_account_id: account_id,
            symbol_id: symbol_id,
            payload_type: Some(2129),
        };

        // Serialize to protobuf bytes
        let mut buf = Vec::new();
        req.encode(&mut buf)?;

        // Create message with payload type prefix
        let msg_id: u16 = 2129; // Application Auth Request ID
        let mut message = Vec::with_capacity(2 + buf.len());
        message.extend_from_slice(&msg_id.to_le_bytes()); // Message ID
        message.extend_from_slice(&buf); // Protobuf payload

        // Send as binary message
        let ws_msg = WsMessage::Binary(message.into());

        let _ = &mut self.ws_write.lock().await.send(ws_msg).await?;

        Ok(())
    }

    pub async fn send_subscribe_spots_request(
        &self,
        account_id: i64,
        symbol_id: Vec<i64>,
        time_in_seconds: usize,
        subscribe_to_spot_timestamp: bool,
    ) -> Result<(), anyhow::Error> {
        let req = ProtoOaSubscribeSpotsReq {
            ctid_trader_account_id: account_id,
            symbol_id: symbol_id,
            subscribe_to_spot_timestamp: Some(subscribe_to_spot_timestamp),
            payload_type: Some(2127),
        };

        // Serialize to protobuf bytes
        let mut buf = Vec::new();
        req.encode(&mut buf)?;

        // Create message with payload type prefix
        let msg_id: u16 = 2127; // Application Auth Request ID
        let mut message = Vec::with_capacity(2 + buf.len());
        message.extend_from_slice(&msg_id.to_le_bytes()); // Message ID
        message.extend_from_slice(&buf); // Protobuf payload

        // Send as binary message
        let ws_msg = WsMessage::Binary(message.into());

        let _ = &mut self.ws_write.lock().await.send(ws_msg).await?;

        Ok(())
    }

    pub async fn send_get_tick_data_request(
        &self,
        account_id: i64,
        days: i64,
        quote_type: ProtoOaQuoteType,
        symbol_id: i64,
        from_timestamp: Option<i64>,
        to_timestamp: Option<i64>,
    ) -> Result<(), anyhow::Error> {
        let req = ProtoOaGetTickDataReq {
            ctid_trader_account_id: account_id,
            symbol_id: symbol_id,
            payload_type: Some(2145),
            r#type: quote_type as i32,
            from_timestamp: from_timestamp,
            to_timestamp: to_timestamp,
        };

        // Serialize to protobuf bytes
        let mut buf = Vec::new();
        req.encode(&mut buf)?;

        // Create message with payload type prefix
        let msg_id: u16 = 2145; // Application Auth Request ID
        let mut message = Vec::with_capacity(2 + buf.len());
        message.extend_from_slice(&msg_id.to_le_bytes()); // Message ID
        message.extend_from_slice(&buf); // Protobuf payload

        // Send as binary message
        let ws_msg = WsMessage::Binary(message.into());

        let _ = &mut self.ws_write.lock().await.send(ws_msg).await?;

        Ok(())
    }

    pub async fn send_get_trendbars_request(
        &self,
        account_id: i64,
        period: i32,
        symbol_id: i64,
        count: u32,
        from_timestamp: i64,
        to_timestamp: i64,
    ) -> Result<(), anyhow::Error> {
        let req = ProtoOaGetTrendbarsReq {
            ctid_trader_account_id: account_id,
            from_timestamp: Some(from_timestamp),
            to_timestamp: Some(to_timestamp),
            payload_type: Some(2137),
            period: period,
            count: Some(count),
            symbol_id: symbol_id,
        };

        // Serialize to protobuf bytes
        let mut buf = Vec::new();
        req.encode(&mut buf)?;

        // Create message with payload type prefix
        let msg_id: u16 = 2137; // Application Auth Request ID
        let mut message = Vec::with_capacity(2 + buf.len());
        message.extend_from_slice(&msg_id.to_le_bytes()); // Message ID
        message.extend_from_slice(&buf); // Protobuf payload

        // Send as binary message
        let ws_msg = WsMessage::Binary(message.into());

        let _ = &mut self.ws_write.lock().await.send(ws_msg).await?;

        Ok(())
    }

    pub async fn send_new_order_request(
        &self,
        account_id: i64,
        symbol_id: i64,
        order_type: ProtoOaOrderType,
        trade_side: ProtoOaTradeSide,
        volume: i64,
        price: Option<f64>,
    ) -> Result<(), anyhow::Error> {
        let mut req = ProtoOaNewOrderReq {
            ctid_trader_account_id: account_id,
            symbol_id: symbol_id,
            order_type: order_type as i32,
            trade_side: trade_side as i32,
            volume: volume,
            payload_type: Some(2106),
            ..Default::default()
        };

        match order_type {
            ProtoOaOrderType::Limit => {
                req.limit_price = price;
            }

            ProtoOaOrderType::Stop => req.stop_price = price,

            ProtoOaOrderType::Market => {}

            ProtoOaOrderType::StopLossTakeProfit => {}

            _ => {}
        };

        // Serialize to protobuf bytes
        let mut buf = Vec::new();
        req.encode(&mut buf)?;

        // Create message with payload type prefix
        let msg_id: u16 = 2137; // Application Auth Request ID
        let mut message = Vec::with_capacity(2 + buf.len());
        message.extend_from_slice(&msg_id.to_le_bytes()); // Message ID
        message.extend_from_slice(&buf); // Protobuf payload

        // Send as binary message
        let ws_msg = WsMessage::Binary(message.into());

        let _ = &mut self.ws_write.lock().await.send(ws_msg).await?;

        Ok(())
    }

    pub async fn send_reconcile_request(&self, account_id: i64) -> Result<(), anyhow::Error> {
        let req = ProtoOaReconcileReq {
            ctid_trader_account_id: account_id,
            payload_type: Some(2124),
            ..Default::default()
        };

        // Serialize to protobuf bytes
        let mut buf = Vec::new();
        req.encode(&mut buf)?;

        // Create message with payload type prefix
        let msg_id: u16 = 2124; // Application Auth Request ID
        let mut message = Vec::with_capacity(2 + buf.len());
        message.extend_from_slice(&msg_id.to_le_bytes()); // Message ID
        message.extend_from_slice(&buf); // Protobuf payload

        // Send as binary message
        let ws_msg = WsMessage::Binary(message.into());

        let _ = &mut self.ws_write.lock().await.send(ws_msg).await?;

        Ok(())
    }

    pub async fn send_close_position_request(
        &self,
        account_id: i64,
        position_id: i64,
        volume: i64,
    ) -> Result<(), anyhow::Error> {
        let req = ProtoOaClosePositionReq {
            ctid_trader_account_id: account_id,
            position_id: position_id,
            volume: volume,
            payload_type: Some(2111),
        };

        // Serialize to protobuf bytes
        let mut buf = Vec::new();
        req.encode(&mut buf)?;

        // Create message with payload type prefix
        let msg_id: u16 = 2111; // Application Auth Request ID
        let mut message = Vec::with_capacity(2 + buf.len());
        message.extend_from_slice(&msg_id.to_le_bytes()); // Message ID
        message.extend_from_slice(&buf); // Protobuf payload

        // Send as binary message
        let ws_msg = WsMessage::Binary(message.into());

        let _ = &mut self.ws_write.lock().await.send(ws_msg).await?;

        Ok(())
    }

    pub async fn send_cancel_order_request(
        &self,
        account_id: i64,
        order_id: i64,
    ) -> Result<(), anyhow::Error> {
        let req = ProtoOaCancelOrderReq {
            ctid_trader_account_id: account_id,
            order_id: order_id,
            payload_type: Some(2108),
        };

        // Serialize to protobuf bytes
        let mut buf = Vec::new();
        req.encode(&mut buf)?;

        // Create message with payload type prefix
        let msg_id: u16 = 2108; // Application Auth Request ID
        let mut message = Vec::with_capacity(2 + buf.len());
        message.extend_from_slice(&msg_id.to_le_bytes()); // Message ID
        message.extend_from_slice(&buf); // Protobuf payload

        // Send as binary message
        let ws_msg = WsMessage::Binary(message.into());

        let _ = &mut self.ws_write.lock().await.send(ws_msg).await?;

        Ok(())
    }

    pub async fn send_deal_offset_list_request(
        &self,
        account_id: i64,
        deal_id: i64,
    ) -> Result<(), anyhow::Error> {
        let req = ProtoOaDealOffsetListReq {
            ctid_trader_account_id: account_id,
            deal_id: deal_id,
            payload_type: Some(2185),
        };

        // Serialize to protobuf bytes
        let mut buf = Vec::new();
        req.encode(&mut buf)?;

        // Create message with payload type prefix
        let msg_id: u16 = 2185; // Application Auth Request ID
        let mut message = Vec::with_capacity(2 + buf.len());
        message.extend_from_slice(&msg_id.to_le_bytes()); // Message ID
        message.extend_from_slice(&buf); // Protobuf payload

        // Send as binary message
        let ws_msg = WsMessage::Binary(message.into());

        let _ = &mut self.ws_write.lock().await.send(ws_msg).await?;

        Ok(())
    }

    pub async fn send_get_position_unrealized_pnl_equest(
        &self,
        account_id: i64,
    ) -> Result<(), anyhow::Error> {
        let req = ProtoOaGetPositionUnrealizedPnLReq {
            ctid_trader_account_id: account_id,
            payload_type: Some(2137),
        };

        // Serialize to protobuf bytes
        let mut buf = Vec::new();
        req.encode(&mut buf)?;

        // Create message with payload type prefix
        let msg_id: u16 = 2137; // Application Auth Request ID
        let mut message = Vec::with_capacity(2 + buf.len());
        message.extend_from_slice(&msg_id.to_le_bytes()); // Message ID
        message.extend_from_slice(&buf); // Protobuf payload

        // Send as binary message
        let ws_msg = WsMessage::Binary(message.into());

        let _ = &mut self.ws_write.lock().await.send(ws_msg).await?;

        Ok(())
    }

    pub async fn send_order_details_request(
        &self,
        account_id: i64,
        order_id: i64,
    ) -> Result<(), anyhow::Error> {
        let req = ProtoOaOrderDetailsReq {
            ctid_trader_account_id: account_id,
            order_id: order_id,
            payload_type: Some(2181),
        };

        // Serialize to protobuf bytes
        let mut buf = Vec::new();
        req.encode(&mut buf)?;

        // Create message with payload type prefix
        let msg_id: u16 = 2181; // Application Auth Request ID
        let mut message = Vec::with_capacity(2 + buf.len());
        message.extend_from_slice(&msg_id.to_le_bytes()); // Message ID
        message.extend_from_slice(&buf); // Protobuf payload

        // Send as binary message
        let ws_msg = WsMessage::Binary(message.into());

        let _ = &mut self.ws_write.lock().await.send(ws_msg).await?;

        Ok(())
    }

    pub async fn send_order_list_by_position_id_request(
        &self,
        account_id: i64,
        position_id: i64,
        from_timestamp: i64,
        to_timestamp: i64,
    ) -> Result<(), anyhow::Error> {
        let req = ProtoOaOrderListByPositionIdReq {
            ctid_trader_account_id: account_id,
            position_id: position_id,
            payload_type: Some(2183),
            from_timestamp: Some(from_timestamp),
            to_timestamp: Some(to_timestamp),
        };

        // Serialize to protobuf bytes
        let mut buf = Vec::new();
        req.encode(&mut buf)?;

        // Create message with payload type prefix
        let msg_id: u16 = 2183; // Application Auth Request ID
        let mut message = Vec::with_capacity(2 + buf.len());
        message.extend_from_slice(&msg_id.to_le_bytes()); // Message ID
        message.extend_from_slice(&buf); // Protobuf payload

        // Send as binary message
        let ws_msg = WsMessage::Binary(message.into());

        let _ = &mut self.ws_write.lock().await.send(ws_msg).await?;

        Ok(())
    }
}
