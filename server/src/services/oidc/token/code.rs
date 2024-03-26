use serde::Deserialize;
use slo::{errors, Result};
use storage::{authcode::AuthCodeStore, connector::ConnectorStore};

use crate::services::oidc::token;

#[derive(Debug, Deserialize)]
pub struct CodeGrantOpts {
    pub code: String,
    pub redirect_uri: String,
    pub code_verifier: Option<String>,
}

pub async fn code_grant<
    C: AuthCodeStore,
    CN: ConnectorStore,
    T: token::Token,
>(
    auth_store: &C,
    connector_store: &CN,
    token_creator: &T,
    opts: &CodeGrantOpts,
) -> Result<token::TokenResponse> {
    Ok(Default::default())
}
