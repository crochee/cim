mod password;

use async_trait::async_trait;
use http::Request;
use serde::Deserialize;

use cim_core::Result;

use crate::models::provider::Provider;

pub use password::PasswordGrant;

#[derive(Debug, Deserialize)]
pub struct GrantTypes {
    pub grant_type: GrantType,
}

#[derive(Debug, Deserialize)]
pub enum GrantType {
    #[serde(rename = "password")]
    Password,
    #[serde(rename = "authorization_code")]
    AuthorizationCode,
    #[serde(rename = "refresh_token")]
    RefreshToken,
}

#[async_trait]
pub trait Token<B> {
    async fn client(req: Request<B>) -> Result<Provider>;
    async fn handle(req: Request<B>) -> Result<()>;
}
