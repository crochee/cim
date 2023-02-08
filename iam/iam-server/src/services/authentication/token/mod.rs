mod password;

use std::collections::HashMap;

use async_trait::async_trait;
use http::Request;
use mockall::automock;
use serde::Deserialize;

use cim_core::Result;

use crate::models::provider::Provider;

pub use password::PasswordGrant;

#[derive(Debug, Deserialize)]
pub struct GrantTypes {
    pub grant_type: Option<GrantType>,
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
pub trait Token {
    async fn handle<F>(
        &self,
        body: &HashMap<String, String>,
        f: F,
    ) -> Result<()>
    where
        F: FnOnce() -> (String, String) + Send;
}
