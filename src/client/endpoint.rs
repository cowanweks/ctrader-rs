/// CTrader endpoints
/// * AUTH_URI - The CTrader authorization URI (<https://openapi.ctrader.com/apps/auth>)
/// * TOKEN_URI - The CTrader tocken aquisition URI (<https://openapi.ctrader.com/apps/token>)
/// * DEMO_HOST_URI - The CTrader Demo Host HTTP URI (<https://demo.ctraderapi.com>)
/// * DEMO_HOST_WS_URI - The CTrader Demo Host HTTPS URI (<wss://demo.ctraderapi.com>)
/// * LIVE_HOST_WS_URI - The CTrader Live Host WSS URI (<wss://live.ctraderapi.com>)
/// * LIVE_HOST_URI - The CTrader Live Host HTTPS URI (<https://demo.ctraderapi.com>)
/// * JSON_PORT - The CTrader Port with support for JSON (5036)
pub struct Endpoints;

#[allow(dead_code)]
impl Endpoints {
    pub const AUTH_URI: &str = "https://openapi.ctrader.com/apps/auth";
    pub const TOKEN_URI: &str = "https://openapi.ctrader.com/apps/token";
    pub const DEMO_HOST_URI: &str = "https://demo.ctraderapi.com";
    pub const DEMO_HOST_WS_URI: &str = "wss://demo.ctraderapi.com";
    pub const LIVE_HOST_WS_URI: &str = "wss://live.ctraderapi.com";
    pub const LIVE_HOST_URI: &str = "https://live.ctraderapi.com";
    pub const JSON_PORT: usize = 5036;
}
