use serde::Deserialize;
use slo::{Result};
use storage::{connector::ConnectorStore, refresh::RefreshTokenStore};

use crate::services::oidc::token;

#[derive(Debug, Deserialize)]
pub struct RefreshGrantOpts {
    pub refresh_token: String,
    pub scope: String,
}

pub async fn refresh_grant<
    C: RefreshTokenStore,
    CN: ConnectorStore,
    T: token::Token,
>(
    _refresh_store: &C,
    _connector_store: &CN,
    _token_creator: &T,
    _opts: &RefreshGrantOpts,
) -> Result<token::TokenResponse> {
    Ok(Default::default())
}
