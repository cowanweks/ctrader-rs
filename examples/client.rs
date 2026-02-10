use config::{Config, ConfigError, Environment, File};
use ctrader_rs::prelude::CTraderClient;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct App {
    pub debug: bool,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CTrader {
    pub is_demo: bool,
    pub app_client_id: String,
    pub app_access_token: String,
    pub app_client_secret: String,
    pub app_redirect_url: String,
    pub refresh_token: String,
}

/// The settings
#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub app: App,
    pub ctrader: CTrader,
}

impl AppConfig {
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "dev".into());

        // Create configuratin based on multiple parameters
        let config = Config::builder()
            // Start off by merging in the "default" configuration file
            .add_source(File::with_name("config/default"))
            .add_source(Environment::with_prefix("APP_").try_parsing(true))
            .add_source(File::with_name(&format!("config/{run_mode}")).required(false));

        let config = config.build()?;

        config.try_deserialize()
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Initialize the logger
    tracing_subscriber::fmt().init();

    // Load the configuration settings
    let config = AppConfig::new()?;

    let (mut ctrader_client, messages_handle, heartbeat_handle) = CTraderClient::new(
        config.ctrader.is_demo,
        config.ctrader.app_client_id,
        config.ctrader.app_access_token,
        config.ctrader.app_client_secret,
        config.ctrader.app_redirect_url,
        config.ctrader.refresh_token,
    )
    .await?;

    ctrader_client.send_application_auth_request().await?;

    // ctrader_client
    //     .send_new_stop_order(1, 41, ProtoOATradeSide::BUY, 100, 50.00)
    //     .await?;

    let _ = tokio::join!(messages_handle, heartbeat_handle);

    Ok(())
}
