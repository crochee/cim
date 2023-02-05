mod mariadb;

use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;

use cim_core::Result;
use serde::Deserialize;

use crate::models::{auth_request::AuthRequest, claim::Claims, ID};

pub use mariadb::MariadbAuthReqs;

pub type DynAuthReqs = Arc<dyn AuthReqsRep + Send + Sync>;

#[automock]
#[async_trait]
pub trait AuthReqsRep {
    async fn create(&self, content: &Content) -> Result<ID>;
    async fn get(&self, id: &str) -> Result<AuthRequest>;
    async fn update(&self, id: &str, opts: &UpdateOpts) -> Result<()>;
    async fn delete(&self, id: &str) -> Result<()>;
}

#[derive(Debug, Deserialize)]
pub struct Content {
    pub client_id: String,
    pub response_types: Vec<String>,
    pub scopes: Vec<String>,
    pub redirect_url: String,
    pub nonce: String,
    pub state: String,
    pub force_approval: bool,
    pub expiry: i64,
    pub logged_in: bool,
    pub claims: Option<Claims>,
    pub hmac_key: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOpts {
    pub logged_in: bool,
    pub claims: Claims,
}
