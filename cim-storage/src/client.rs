use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(
    Debug, Default, Deserialize, Serialize, ToSchema, PartialEq, Clone,
)]
pub struct Client {
    pub id: String,
    pub secret: String,
    pub redirect_uris: Vec<String>,
    pub trusted_peers: Vec<String>,
    pub name: String,
    pub logo_url: String,
    pub account_id: String,

    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
