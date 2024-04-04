mod mariadb;

use async_trait::async_trait;
use chrono::NaiveDateTime;
use mockall::automock;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use utoipa::ToSchema;
use validator::Validate;

use slo::Result;

use crate::{Claim, ID};

pub use mariadb::AuthRequestImpl;

#[derive(Debug, Deserialize, Default, Serialize, Validate, ToSchema)]
pub struct AuthRequest {
    pub id: String,
    pub client_id: String,
    pub response_types: Vec<String>,
    pub scopes: Vec<String>,
    pub redirect_uri: String,
    pub code_challenge: String,
    pub code_challenge_method: String,
    pub nonce: String,
    pub state: String,
    pub hmac_key: String,
    pub force_approval_prompt: bool,
    pub logged_in: bool,

    #[serde(flatten)]
    pub claim: Claim,

    pub connector_id: String,
    pub connector_data: Option<Box<RawValue>>,

    pub expiry: i64,

    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[automock]
#[async_trait]
pub trait AuthRequestStore {
    async fn put_auth_request(&self, content: &AuthRequest) -> Result<ID>;
    async fn get_auth_request(&self, id: &str) -> Result<AuthRequest>;
    async fn delete_auth_request(&self, id: &str) -> Result<()>;
}
