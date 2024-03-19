use async_trait::async_trait;
use chrono::NaiveDateTime;
use mockall::automock;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use slo::Result;

use crate::ID;

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

    pub claims_user_id: String,
    pub claims_user_name: String,
    pub claims_email: String,
    pub claims_email_verified: bool,
    pub claims_groups: String,
    pub claims_preferred_username: String,

    pub connector_id: String,
    pub connector_data: Option<String>,

    pub expires_in: i64,

    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct Content {
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

    pub claims_user_id: String,
    pub claims_user_name: String,
    pub claims_email: String,
    pub claims_email_verified: bool,
    pub claims_groups: String,
    pub claims_preferred_username: String,

    pub connector_id: String,
    pub connector_data: Option<String>,

    pub expires_in: i64,
}

#[automock]
#[async_trait]
pub trait AuthRequestStore {
    async fn put_auth_request(
        &self,
        id: Option<String>,
        content: &Content,
    ) -> Result<ID>;
    async fn get_auth_request(&self, id: &str) -> Result<AuthRequest>;
    async fn delete_auth_request(&self, id: &str) -> Result<()>;
}
