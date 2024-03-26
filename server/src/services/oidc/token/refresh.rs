use serde::Deserialize;
use slo::{errors, Result};
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
    refresh_store: &C,
    connector_store: &CN,
    token_creator: &T,
    opts: &RefreshGrantOpts,
) -> Result<token::TokenResponse> {
    Ok(Default::default())
}
