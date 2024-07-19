use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use utoipa::ToSchema;

use crate::Claim;

#[derive(Debug, Default, Deserialize, Serialize, ToSchema, Clone)]
pub struct RefreshToken {
    pub id: String,
    pub client_id: String,
    pub scopes: Vec<String>,
    pub nonce: String,
    pub token: String,
    pub obsolete_token: String,

    #[serde(flatten)]
    pub claim: Claim,

    pub connector_id: String,
    pub connector_data: Option<Box<RawValue>>,

    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub last_used_at: NaiveDateTime,
}

impl PartialEq for RefreshToken {
    fn eq(&self, other: &Self) -> bool {
        if self.id == other.id
            && self.client_id == other.client_id
            && self.scopes == other.scopes
            && self.nonce == other.nonce
            && self.token == other.token
            && self.obsolete_token == other.obsolete_token
            && self.claim == other.claim
            && self.connector_id == other.connector_id
            && self.created_at == other.created_at
            && self.updated_at == other.updated_at
            && self.last_used_at == other.last_used_at
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
