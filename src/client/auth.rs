use super::endpoint::Endpoints;
use crate::{
    error::CTraderResult,
    types::{Auth, TokenResponse},
};

impl Auth {
    pub fn new(
        app_client_id: String,
        app_access_token: String,
        app_client_secret: String,
        app_redirect_url: String,
        refresh_token: String,
    ) -> Self {
        Self {
            ctrader_access_token: app_access_token,
            ctrader_refresh_token: refresh_token,
            ctrader_client_id: app_client_id,
            ctrader_client_secret: app_client_secret,
            ctrader_redirect_url: app_redirect_url,
        }
    }

    pub fn get_auth_uri(self) -> CTraderResult<String> {
        let auth_uri = format!(
            "{}?client_id={}&redirect_uri={}&scope=trading",
            Endpoints::AUTH_URI,
            self.ctrader_client_id,
            self.ctrader_redirect_url
        );
        Ok(auth_uri)
    }

    pub async fn get_token(self, auth_code: &str) -> CTraderResult<TokenResponse> {
        let url = format!(
            "{}?grant_type=authorization_code&code={}&redirect_uri={}&client_id={}&client_secret={}",
            Endpoints::TOKEN_URI,
            auth_code,
            self.ctrader_redirect_url,
            self.ctrader_client_id,
            self.ctrader_client_secret
        );
        let response = reqwest::Client::new()
            .get(url)
            .send()
            .await
            .expect("unable to get access token");

        let res = response
            .json::<TokenResponse>()
            .await
            .expect("error decoding json response!");

        Ok(res)
    }

    pub async fn refresh_token(self, refresh_token: &str) -> CTraderResult<TokenResponse> {
        let url = format!(
            "{}?grant_type=refresh_token&refresh_token={}&client_id={}&client_secret={}",
            Endpoints::TOKEN_URI,
            refresh_token,
            self.ctrader_client_id,
            self.ctrader_client_secret
        );
        let response = reqwest::Client::new()
            .get(url)
            .send()
            .await
            .expect("unable to refresh access token");

        let res = response
            .json::<TokenResponse>()
            .await
            .expect("error decoding json response!");

        Ok(res)
    }
}
