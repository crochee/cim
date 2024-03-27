use async_trait::async_trait;
use chrono::NaiveDateTime;
use mockall::automock;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use slo::Result;

use crate::{Claim, ID};

#[derive(Debug, Clone, Deserialize, Default, Serialize, Validate, ToSchema)]
pub struct RefreshToken {
    pub id: String,
    pub client_id: String,
    pub scopes: Vec<String>,
    pub nonce: String,
    pub token: String,

    #[serde(flatten)]
    pub claim: Claim,

    pub connector_id: String,
    pub connector_data: Option<String>,

    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[automock]
#[async_trait]
pub trait RefreshTokenStore {
    async fn put_refresh_token(&self, content: &RefreshToken) -> Result<ID>;
    async fn get_refresh_token(&self, id: &str) -> Result<RefreshToken>;
    async fn delete_refresh_token(&self, id: &str) -> Result<()>;
}
