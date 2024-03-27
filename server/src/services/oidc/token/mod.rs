pub mod code;
pub mod password;
pub mod refresh;
mod tokenx;

use async_trait::async_trait;
use constant_time_eq::constant_time_eq;
use mockall::automock;
use serde::{Deserialize, Serialize};

use slo::{errors, Result};

use storage::{
    client::{Client, ClientStore},
    Claim,
};
pub use tokenx::AccessToken;

#[automock]
#[async_trait]
pub trait Token {
    async fn token(&self, claims: &Claims) -> Result<(String, i64)>;
    async fn verify(&self, token: &str) -> Result<Claims>;
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Claims {
    pub aud: String, // Optional. Audience
    pub exp: i64, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    pub nbf: i64, // Optional. Not Before (as UTC timestamp)
    pub iss: String, // Optional. Issuer

    pub nonce: String,
    pub access_token: Option<String>,

    #[serde(flatten)]
    pub claim: Claim,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<i64>,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    pub scopes: Option<Vec<String>>,
}

pub const GRANT_TYPE_AUTHORIZATION_CODE: &str = "authorization_code";
pub const GRANT_TYPE_REFRESH_TOKEN: &str = "refresh_token";
pub const GRANT_TYPE_PASSWORD: &str = "password";

pub async fn get_client_and_valid<C: ClientStore>(
    client_store: &C,
    client_id: &str,
    client_secret: &str,
) -> Result<Client> {
    let client = client_store.get_client(client_id, None).await?;
    if !constant_time_eq(client.secret.as_bytes(), client_secret.as_bytes()) {
        return Err(errors::unauthorized());
    }
    Ok(client)
}
