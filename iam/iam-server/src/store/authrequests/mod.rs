mod mariadb;

use serde::Deserialize;

use crate::models::claim::Claims;

pub use mariadb::*;

#[derive(Debug, Deserialize)]
pub struct Content {
    pub client_id: String,
    pub response_types: Vec<String>,
    pub scopes: Vec<String>,
    pub redirect_url: String,
    pub nonce: String,
    pub state: String,
    pub force_approval: bool,
    pub expiry: i64,
    pub logged_in: bool,
    pub claims: Option<Claims>,
    pub hmac_key: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOpts {
    pub logged_in: bool,
    pub claims: Claims,
}
