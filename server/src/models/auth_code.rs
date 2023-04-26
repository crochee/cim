use serde::{Deserialize, Serialize};

use super::claim::Claims;

#[derive(Debug, Deserialize, Serialize)]
pub struct AuthCode {
    pub id: String,
    pub client_id: String,
    pub redirect_url: String,
    pub nonce: String,
    pub scope: String,
    pub expiry: i64,
    pub claims: Claims,
}
