use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

use crate::Claim;

#[derive(Debug, Default, Deserialize, Serialize, Clone, utoipa::ToSchema)]
pub struct AuthCode {
    pub id: String,
    pub client_id: String,
    pub scopes: Vec<String>,
    pub nonce: String,
    pub redirect_uri: String,
    pub code_challenge: String,
    pub code_challenge_method: String,

    #[serde(flatten)]
    pub claim: Claim,

    pub connector_id: String,
    #[schema(format = Binary, value_type = String)]
    pub connector_data: Option<Box<RawValue>>,

    pub expiry: i64,

    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl PartialEq for AuthCode {
    fn eq(&self, other: &Self) -> bool {
        if self.id == other.id
            && self.client_id == other.client_id
            && self.scopes == other.scopes
            && self.nonce == other.nonce
            && self.redirect_uri == other.redirect_uri
            && self.code_challenge == other.code_challenge
            && self.code_challenge_method == other.code_challenge_method
            && self.claim == other.claim
            && self.connector_id == other.connector_id
            && self.expiry == other.expiry
            && self.created_at == other.created_at
            && self.updated_at == other.updated_at
        {
            if let Some(connector_data) = &self.connector_data {
                if let Some(other_connector_data) = &other.connector_data {
                    return connector_data.get() == other_connector_data.get();
                }
            }
        }
        false
    }
}
