mod mariadb;

use std::collections::HashMap;

use async_trait::async_trait;
use chrono::NaiveDateTime;
use mockall::automock;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use utoipa::ToSchema;
use validator::Validate;

use slo::Result;

pub use mariadb::OfflineSessionImpl;

#[derive(Debug, Clone, Deserialize, Default, Serialize, Validate, ToSchema)]
pub struct OfflineSession {
    pub user_id: String,
    pub conn_id: String,
    pub refresh: HashMap<String, RefreshTokenRef>,

    pub connector_data: Option<Box<RawValue>>,
}

#[derive(Debug, Clone, Deserialize, Default, Serialize, Validate, ToSchema)]
pub struct RefreshTokenRef {
    pub id: String,
    pub client_id: String,
    pub created_at: NaiveDateTime,
    pub last_used_at: NaiveDateTime,
}
#[automock]
#[async_trait]
pub trait OfflineSessionStore {
    async fn put_offline_session(&self, content: &OfflineSession)
        -> Result<()>;
    async fn get_offline_session(
        &self,
        user_id: &str,
        conn_id: &str,
    ) -> Result<OfflineSession>;
    async fn delete_offline_session(
        &self,
        user_id: &str,
        conn_id: &str,
    ) -> Result<()>;
}
