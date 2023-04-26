use serde::{Deserialize, Serialize};

use super::claim::Claims;

#[derive(Debug, Deserialize, Serialize)]
pub struct AuthRequest {
    pub id: String,
    pub client_id: String,
    pub response_types: Vec<String>,
    pub scopes: Vec<String>,
    pub redirect_url: String,
    pub nonce: String,
    pub state: String,
    /// The client has indicated that the end user must be shown an approval prompt on all requests. The server cannot cache their initial action for subsequent attempts.
    pub force_approval: bool,
    pub expiry: i64,
    /// Has the user proved their identity through a backing identity provider?
    ///
    /// If false, the following fields are invalid.
    pub logged_in: bool,
    pub claims: Option<Claims>,
    /// HMACKey is used when generating an AuthRequest-specific
    pub hmac_key: String,
}
