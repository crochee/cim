use async_trait::async_trait;
use chrono::NaiveDateTime;
use mockall::automock;
use serde::{Deserialize, Serialize};
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
    pub connector_data: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct Content {
    pub secret: String,
    pub redirect_uris: Vec<String>,
    pub trusted_peers: Vec<String>,
    pub name: String,
    pub logo_url: String,
}

#[automock]
#[async_trait]
pub trait ConnectorStore {
    async fn put_connector(
        &self,
        id: Option<String>,
        content: &Content,
    ) -> Result<ID>;
    async fn get_connector(&self, id: &str) -> Result<Connector>;
    async fn delete_connector(&self, id: &str) -> Result<()>;
}
