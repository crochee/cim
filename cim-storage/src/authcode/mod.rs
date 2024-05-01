mod mariadb;

use async_trait::async_trait;
use chrono::NaiveDateTime;
use mockall::automock;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use utoipa::ToSchema;
use validator::Validate;

use cim_slo::Result;

use crate::{Claim, ID};

pub use mariadb::AuthCodeImpl;

#[derive(Debug, Clone, Deserialize, Default, Serialize, Validate, ToSchema)]
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
    pub connector_data: Option<Box<RawValue>>,

    pub expiry: i64,

    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[automock]
#[async_trait]
pub trait AuthCodeStore {
    async fn put_auth_code(&self, content: &AuthCode) -> Result<ID>;
    async fn get_auth_code(&self, id: &str) -> Result<AuthCode>;
    async fn delete_auth_code(&self, id: &str) -> Result<()>;
}
