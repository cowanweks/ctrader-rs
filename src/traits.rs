use crate::proto_messages::OpenApiModelMessages::{ProtoOAOrderType, ProtoOATradeSide};

pub trait AppState {}

/// The CTrader OpenAPI supported messages
#[allow(async_fn_in_trait)]
pub trait OpenAPIMessage {
    /// Send a refresh token request
    async fn send_refresh_token_request(&self) -> Result<(), anyhow::Error>;

    /// Send a account auth request
    async fn send_application_auth_request(&mut self) -> Result<(), anyhow::Error>;

    /// Send a set account request to make account active for the next requests
    /// * account_id - The id of the account account to make active
    async fn send_set_account_request(&mut self, account_id: i64) -> Result<(), anyhow::Error>;

    /// Send a get account list request
    async fn send_get_account_list_by_access_token_request(&self) -> Result<(), anyhow::Error>;

    /// Send a account logout request
    async fn send_account_logout_request(&self, account_id: i64) -> Result<(), anyhow::Error>;

    /// Send a asset list request
    async fn send_asset_list_request(&self, account_id: i64) -> Result<(), anyhow::Error>;

    /// Send a asset class list request
    async fn send_asset_class_list_request(&self, account_id: i64) -> Result<(), anyhow::Error>;

    /// Send a symbol category list request
    async fn send_symbol_category_list_request(&self, account_id: i64)
    -> Result<(), anyhow::Error>;

    /// Send a symbol list request
    /// * include_archived_symbols - Whether to include archived symbols in the response
    async fn send_symbols_list_request(
        &self,
        account_id: i64,
        include_archived_symbols: bool,
    ) -> Result<(), anyhow::Error>;

    /// Send a trader request
    async fn send_trader_request(&self, account_id: i64) -> Result<(), anyhow::Error>;

    /// Send a unsubscribe spots request
    /// * symbol_id - The id of the symbol to unsubscribe spots
    async fn send_unsubscribe_spots_request(
        &self,
        account_id: i64,
        symbol_id: Vec<i64>,
    ) -> Result<(), anyhow::Error>;

    /// Send a subscribe spots request
    /// * symbol_id - The id of the symbol to subscribe spots
    /// * time_in_seconds - The time in seconds to subscribe
    /// * subscribe_to_spot_timestamp - Whether to subscribe to spot timestamp
    async fn send_subscribe_spots_request(
        &self,
        account_id: i64,
        symbol_id: Vec<i64>,
        time_in_seconds: usize,
        subscribe_to_spot_timestamp: bool,
    ) -> Result<(), anyhow::Error>;

    /// Send a reconcile request
    async fn send_reconcile_request(&self, account_id: i64) -> Result<(), anyhow::Error>;

    /// Send a get trendbars request
    /// * weeks - Number of weeks to get trendbars for
    /// * period - The period for the trendbars
    /// * symbol_id - The id of the symbol to get the trendbars
    async fn send_get_trendbars_request(
        &self,
        account_id: i64,
        weeks: i64,
        period: i64,
        symbol_id: i64,
    ) -> Result<(), anyhow::Error>;

    /// Send a get tick data request
    /// * days - The number of days to get tick data for
    /// * quote_type - The quote type to get tick data for
    /// * symbol_id - The id of the symbol to get tick data for
    async fn send_get_tick_data_request(
        &self,
        account_id: i64,
        days: i64,
        quote_type: &str,
        symbol_id: i64,
    ) -> Result<(), anyhow::Error>;

    /// Send a new order request
    /// * symbol_id - The id of the symbol to place order for
    /// * order_type - The type of the order to place
    /// * trade_side - The Direction of the trade to be placed
    /// * volume - The amount of the volume to place the order for
    /// * price - The price
    async fn send_new_order_request(
        &self,
        account_id: i64,
        symbol_id: i64,
        order_type: ProtoOAOrderType,
        trade_side: ProtoOATradeSide,
        volume: i64,
        price: Option<f64>,
    ) -> Result<(), anyhow::Error>;

    /// Send a close position request
    /// * position_id - The id of the position to close
    /// * volume - The amount in volume for the position to close
    async fn send_close_position_request(
        &self,
        account_id: i64,
        position_id: i64,
        volume: i64,
    ) -> Result<(), anyhow::Error>;

    /// Send a cancel order request
    /// * order_id - The id of the order to cancel
    async fn send_cancel_order_request(
        &self,
        account_id: i64,
        order_id: i64,
    ) -> Result<(), anyhow::Error>;

    /// Send a deal offset list request
    /// * deal_id - The id of the deal to get the offset list for
    async fn send_deal_offset_list_request(
        &self,
        account_id: i64,
        deal_id: i64,
    ) -> Result<(), anyhow::Error>;

    /// Send a get position unrealized PNL request
    async fn send_get_position_unrealized_pnl_equest(
        &self,
        account_id: i64,
    ) -> Result<(), anyhow::Error>;

    /// Send a order details request
    /// * order_id - The id of the order to get order details for
    async fn send_order_details_request(
        &self,
        account_id: i64,
        order_id: i64,
    ) -> Result<(), anyhow::Error>;

    /// Send a order list by position id request
    async fn send_order_list_by_position_id_request(
        &self,
        account_id: i64,
    ) -> Result<(), anyhow::Error>;
}
