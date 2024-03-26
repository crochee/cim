use constant_time_eq::constant_time_eq;
use serde::Deserialize;
use slo::{errors, Result};
use storage::client::{Client, ClientStore};
use validator::Validate;

use super::TokenResponse;

#[derive(Debug, Deserialize, Validate)]
pub struct GrantOpts {
    pub grant_type: String,
    pub scope: String,
    pub audience: String,
    pub nonce: Option<String>,
}

pub async fn handle_token<C: ClientStore>(
    client_store: &C,
    client_id: &str,
    client_secret: &str,
    opts: &GrantOpts,
) -> Result<TokenResponse> {
    todo!()
}

pub async fn handle_with_client<
    C: ClientStore,
    F: FnOnce(&Client) -> Result<TokenResponse>,
>(
    client_store: &C,
    client_id: &str,
    client_secret: &str,
    f: F,
) -> Result<TokenResponse> {
    let client = client_store.get_client(client_id, None).await?;
    if !constant_time_eq(client.secret.as_bytes(), client_secret.as_bytes()) {
        return Err(errors::unauthorized());
    }
    f(&client)
}
