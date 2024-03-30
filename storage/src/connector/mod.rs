use async_trait::async_trait;
use mockall::automock;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use utoipa::ToSchema;
use validator::Validate;

use slo::Result;

use crate::ID;

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
pub struct Connector {
    pub id: String,
    #[serde(rename = "type")]
    pub connector_type: String,
    pub name: String,
    pub response_version: String,
    pub config: String,
    pub connector_data: Option<Box<RawValue>>,
}

#[automock]
#[async_trait]
pub trait ConnectorStore {
    async fn put_connector(&self, content: &Connector) -> Result<ID>;
    async fn get_connector(&self, id: &str) -> Result<Connector>;
    async fn delete_connector(&self, id: &str) -> Result<()>;
}
