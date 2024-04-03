mod mariadb;

use async_trait::async_trait;
use chrono::NaiveDateTime;
use mockall::automock;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use slo::Result;

use crate::ID;

pub use mariadb::ClientImpl;

#[derive(Debug, Default, Deserialize, Serialize, Validate, ToSchema)]
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

#[automock]
#[async_trait]
pub trait ClientStore {
    async fn put_client(&self, content: &Client) -> Result<ID>;
    async fn get_client(
        &self,
        id: &str,
        account_id: Option<String>,
    ) -> Result<Client>;
    async fn delete_client(
        &self,
        id: &str,
        account_id: Option<String>,
    ) -> Result<()>;
}
