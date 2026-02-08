mod auth;
mod connector;
mod endpoint;
mod middleware;
mod receiver;
mod sender;

use crate::client::{receiver::on_message, sender::send_heartbeat};
use crate::proto_messages::OpenApiMessages::ProtoOAApplicationAuthReq;
use crate::proto_messages::OpenApiModelMessages::{ProtoOAOrderType, ProtoOATradeSide};
use endpoint::Endpoints;

use crate::{
    traits::OpenAPIMessage,
    types::{Auth, CTraderClient},
};

use futures_util::{SinkExt, StreamExt};

use std::time::Duration;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio_tungstenite::{connect_async, tungstenite::Message};

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

        let url = format!("{}:{}", host, Endpoints::JSON_PORT);

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
        trade_side: ProtoOATradeSide,
        volume: i64,
        price: f64,
    ) -> Result<(), anyhow::Error> {
        let order_type = ProtoOAOrderType::LIMIT;

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
        trade_side: ProtoOATradeSide,
        volume: i64,
    ) -> Result<(), anyhow::Error> {
        let order_type = ProtoOAOrderType::MARKET;

        self.send_new_order_request(account_id, symbol_id, order_type, trade_side, volume, None)
            .await?;
        Ok(())
    }

    /// Send a new STOP order request
    pub async fn send_new_stop_order(
        &self,
        account_id: i64,
        symbol_id: i64,
        trade_side: ProtoOATradeSide,
        volume: i64,
        price: f64,
    ) -> Result<(), anyhow::Error> {
        let order_type = ProtoOAOrderType::STOP;

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
}

impl OpenAPIMessage for CTraderClient {
    async fn send_refresh_token_request(&self) -> Result<(), anyhow::Error> {
        tracing::info!("Refreshing Application Token");

        let req = serde_json::json!({
            "clientMsgId": "cm_id_2",
            "payloadType" : 2173,
            "payload": {
                "refreshToken": self.auth.ctrader_refresh_token,
            }
        });

        let msg = Message::Text(req.to_string().into());

        self.ws_write.lock().await.send(msg).await?;

        Ok(())
    }

    /// Authenticate the client to the CTrader APi
    async fn send_application_auth_request(&mut self) -> Result<(), anyhow::Error> {
        tracing::info!("Authenticating Async Client to CTrader OpenAPI");

        let req = serde_json::json!({
            "payloadType" : 2100,
            "payload": {
                "clientId": self.auth.ctrader_client_id,
                "clientSecret": self.auth.ctrader_client_secret
            }
        });

        // Convert JSON to Message
        let msg = Message::Text(req.to_string().into());

        println!("{:?}", msg);

        let _ = &mut self.ws_write.lock().await.send(msg).await?;

        Ok(())
    }

    async fn send_set_account_request(&mut self, account_id: i64) -> Result<(), anyhow::Error> {
        tracing::info!("Setting Active account to {}", account_id);

        let req = serde_json::json!({
            "clientMsgId": "cm_id_2",
            "payloadType" : 2100,
            "payload": {
                "ctidTraderAccountId": account_id,
                "accessToken": self.auth.ctrader_access_token,
            }
        });

        let msg = Message::Text(req.to_string().into());

        self.ws_write.lock().await.send(msg).await?;

        Ok(())
    }

    async fn send_get_account_list_by_access_token_request(&self) -> Result<(), anyhow::Error> {
        let req = serde_json::json!({
            "clientMsgId": "cm_id_2",
            "payloadType" : 2149,
            "payload": {
                "accessToken": "kjo8CTjHiSWGvx8t9ASkCn9SZccvVr7U__yliip4260",
            }
        });

        // Convert JSON to Message
        let msg = Message::Text(req.to_string().into());

        let _ = &mut self.ws_write.lock().await.send(msg).await?;

        Ok(())
    }

    async fn send_account_logout_request(&self, account_id: i64) -> Result<(), anyhow::Error> {
        let req = serde_json::json!({
            "clientMsgId": "cm_id_2",
            "payloadType" : 2162,
            "payload": {
                "ctidTraderAccountId": account_id,
            }
        });

        // Convert JSON to Message
        let msg = Message::Text(req.to_string().into());

        let _ = &mut self.ws_write.lock().await.send(msg).await?;

        Ok(())
    }

    async fn send_asset_list_request(&self, account_id: i64) -> Result<(), anyhow::Error> {
        let req = serde_json::json!({
            "clientMsgId": "cm_id_2",
            "payloadType" : 2112,
            "payload": {
                "ctidTraderAccountId": account_id,
            }
        });

        // Convert JSON to Message
        let msg = Message::Text(req.to_string().into());

        let _ = &mut self.ws_write.lock().await.send(msg).await?;

        Ok(())
    }

    async fn send_asset_class_list_request(&self, account_id: i64) -> Result<(), anyhow::Error> {
        let req = serde_json::json!({
            "clientMsgId": "cm_id_2",
            "payloadType" : 2153,
            "payload": {
                "ctidTraderAccountId": account_id,
            }
        });

        // Convert JSON to Message
        let msg = Message::Text(req.to_string().into());

        let _ = &mut self.ws_write.lock().await.send(msg).await?;

        Ok(())
    }

    async fn send_symbol_category_list_request(
        &self,
        account_id: i64,
    ) -> Result<(), anyhow::Error> {
        let req = serde_json::json!({
            "clientMsgId": "cm_id_2",
            "payloadType" : 2160,
            "payload": {
                "ctidTraderAccountId": account_id,
            }
        });

        // Convert JSON to Message
        let msg = Message::Text(req.to_string().into());

        let _ = &mut self.ws_write.lock().await.send(msg).await?;

        Ok(())
    }

    async fn send_symbols_list_request(
        &self,
        account_id: i64,
        include_archived_symbols: bool,
    ) -> Result<(), anyhow::Error> {
        let req = serde_json::json!({
            "clientMsgId": "cm_id_2",
            "payloadType" : 2114,
            "payload": {
                "ctidTraderAccountId": account_id,
                "includeArchivedSymbols": include_archived_symbols
            }
        });

        // Convert JSON to Message
        let msg = Message::Text(req.to_string().into());

        let _ = &mut self.ws_write.lock().await.send(msg).await?;

        Ok(())
    }

    async fn send_trader_request(&self, account_id: i64) -> Result<(), anyhow::Error> {
        let req = serde_json::json!({
            "clientMsgId": "cm_id_2",
            "payloadType" : 2121,
            "payload": {
                "ctidTraderAccountId": account_id,
            }
        });

        // Convert JSON to Message
        let msg = Message::Text(req.to_string().into());
        let _ = &mut self.ws_write.lock().await.send(msg).await?;

        Ok(())
    }

    async fn send_unsubscribe_spots_request(
        &self,
        account_id: i64,
        symbol_id: Vec<i64>,
    ) -> Result<(), anyhow::Error> {
        let req = serde_json::json!({
            "clientMsgId": "cm_id_2",
            "payloadType" : 2129,
            "payload": {
                "ctidTraderAccountId": account_id,
                "symbolId": symbol_id
            }
        });

        // Convert JSON to Message
        let msg = Message::Text(req.to_string().into());
        let _ = &mut self.ws_write.lock().await.send(msg).await?;

        Ok(())
    }

    async fn send_subscribe_spots_request(
        &self,
        account_id: i64,
        symbol_id: Vec<i64>,
        time_in_seconds: usize,
        subscribe_to_spot_timestamp: bool,
    ) -> Result<(), anyhow::Error> {
        let req = serde_json::json!({
            "clientMsgId": "cm_id_2",
            "payloadType" : 2127,
            "payload": {
                "ctidTraderAccountId": account_id,
                "symbolId": symbol_id,
                "subscribeToSpotTimestamp": subscribe_to_spot_timestamp
            }
        });

        // Convert JSON to Message
        let msg = Message::Text(req.to_string().into());
        let _ = &mut self.ws_write.lock().await.send(msg).await?;

        Ok(())
    }

    async fn send_get_tick_data_request(
        &self,
        account_id: i64,
        days: i64,
        quote_type: &str,
        symbol_id: i64,
    ) -> Result<(), anyhow::Error> {
        let req = serde_json::json!({
            "clientMsgId": "cm_id_2",
            "payloadType" : 2145,
            "payload": {
                "ctidTraderAccountId": account_id,
                "symbolId": symbol_id,
            }
        });

        // Convert JSON to Message
        let msg = Message::Text(req.to_string().into());
        let _ = &mut self.ws_write.lock().await.send(msg).await?;

        Ok(())
    }

    async fn send_get_trendbars_request(
        &self,
        account_id: i64,
        weeks: i64,
        period: i64,
        symbol_id: i64,
    ) -> Result<(), anyhow::Error> {
        let req = serde_json::json!({
            "clientMsgId": "cm_id_2",
            "payloadType" : 2137,
            "payload": {
                "ctidTraderAccountId": account_id,
                "symbolId": symbol_id,
            }
        });

        // Convert JSON to Message
        let msg = Message::Text(req.to_string().into());
        let _ = &mut self.ws_write.lock().await.send(msg).await?;

        Ok(())
    }

    async fn send_new_order_request(
        &self,
        account_id: i64,
        symbol_id: i64,
        order_type: ProtoOAOrderType,
        trade_side: ProtoOATradeSide,
        volume: i64,
        price: Option<f64>,
    ) -> Result<(), anyhow::Error> {
        let mut payload = HashMap::new();

        payload.insert("ctidTraderAccountId", account_id.to_string());
        payload.insert("symbolId", symbol_id.to_string());
        payload.insert("volume", volume.to_string());

        match trade_side {
            ProtoOATradeSide::BUY => {
                payload.insert("tradeSide", "BUY".to_string());
            }
            ProtoOATradeSide::SELL => {
                payload.insert("tradeSide", "SELL".to_string());
            }
            _ => {}
        }

        match order_type {
            ProtoOAOrderType::LIMIT => {
                payload.insert("orderType", "LIMIT".to_string());
            }

            ProtoOAOrderType::STOP => {
                payload.insert("orderType", "STOP".to_string());
            }

            ProtoOAOrderType::MARKET => {
                payload.insert("orderType", "MARKET".to_string());
            }

            _ => {}
        }

        match order_type {
            ProtoOAOrderType::LIMIT => {
                payload.insert("limitPrice", price.unwrap().to_string());
            }

            ProtoOAOrderType::STOP => {
                payload.insert("stopPrice", price.unwrap().to_string());
            }

            ProtoOAOrderType::MARKET => {}

            _ => {}
        };

        let req = serde_json::json!({
            "clientMsgId": "cm_id_2",
            "payloadType" : 2106,
            "payload": payload
        });

        // Convert JSON to Message
        let msg = Message::Text(req.to_string().into());
        let _ = &mut self.ws_write.lock().await.send(msg).await?;

        Ok(())
    }

    async fn send_reconcile_request(&self, account_id: i64) -> Result<(), anyhow::Error> {
        let req = serde_json::json!({
            "clientMsgId": "cm_id_2",
            "payloadType" : 2124,
            "payload": {
                "ctidTraderAccountId": account_id
            }
        });

        // Convert JSON to Message
        let msg = Message::Text(req.to_string().into());
        let _ = &mut self.ws_write.lock().await.send(msg).await?;

        Ok(())
    }

    async fn send_close_position_request(
        &self,
        account_id: i64,
        position_id: i64,
        volume: i64,
    ) -> Result<(), anyhow::Error> {
        let req = serde_json::json!({
            "clientMsgId": "cm_id_2",
            "payloadType" : 2111,
            "payload": {
                "ctidTraderAccountId": account_id,
                "positionId": position_id,
                "volume": volume
            }
        });

        // Convert JSON to Message
        let msg = Message::Text(req.to_string().into());
        let _ = &mut self.ws_write.lock().await.send(msg).await?;

        Ok(())
    }

    async fn send_cancel_order_request(
        &self,
        account_id: i64,
        order_id: i64,
    ) -> Result<(), anyhow::Error> {
        let req = serde_json::json!({
            "clientMsgId": "cm_id_2",
            "payloadType" : 2108,
            "payload": {
                "ctidTraderAccountId": account_id,
                "orderId": order_id
            }
        });

        // Convert JSON to Message
        let msg = Message::Text(req.to_string().into());
        let _ = &mut self.ws_write.lock().await.send(msg).await?;

        Ok(())
    }

    async fn send_deal_offset_list_request(
        &self,
        account_id: i64,
        deal_id: i64,
    ) -> Result<(), anyhow::Error> {
        let req = serde_json::json!({
            "clientMsgId": "cm_id_2",
            "payloadType" : 2185,
            "payload": {
                "ctidTraderAccountId": account_id,
                "orderId": deal_id
            }
        });

        // Convert JSON to Message
        let msg = Message::Text(req.to_string().into());
        let _ = &mut self.ws_write.lock().await.send(msg).await?;

        Ok(())
    }

    async fn send_get_position_unrealized_pnl_equest(
        &self,
        account_id: i64,
    ) -> Result<(), anyhow::Error> {
        let req = serde_json::json!({
            "clientMsgId": "cm_id_2",
            "payloadType" : 2187,
            "payload": {
                "ctidTraderAccountId": account_id
            }
        });

        // Convert JSON to Message
        let msg = Message::Text(req.to_string().into());
        let _ = &mut self.ws_write.lock().await.send(msg).await?;

        Ok(())
    }

    async fn send_order_details_request(
        &self,
        account_id: i64,
        order_id: i64,
    ) -> Result<(), anyhow::Error> {
        let req = serde_json::json!({
            "clientMsgId": "cm_id_2",
            "payloadType" : 2181,
            "payload": {
                "ctidTraderAccountId": account_id,
                "orderId": order_id
            }
        });

        // Convert JSON to Message
        let msg = Message::Text(req.to_string().into());
        let _ = &mut self.ws_write.lock().await.send(msg).await?;

        Ok(())
    }

    async fn send_order_list_by_position_id_request(
        &self,
        account_id: i64,
    ) -> Result<(), anyhow::Error> {
        let req = serde_json::json!({
            "clientMsgId": "cm_id_2",
            "payloadType" : 2183,
            "payload": {
                "ctidTraderAccountId": account_id
            }
        });

        // Convert JSON to Message
        let msg = Message::Text(req.to_string().into());
        let _ = &mut self.ws_write.lock().await.send(msg).await?;

        Ok(())
    }
}
