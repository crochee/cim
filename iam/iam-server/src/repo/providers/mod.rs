mod mariadb;

use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;

use cim_core::Result;
use serde::Deserialize;

use crate::models::{provider::Provider, ID};

pub use mariadb::MariadbProviders;

pub type DynProviders = Arc<dyn ProvidersRep + Send + Sync>;

#[automock]
#[async_trait]
pub trait ProvidersRep {
    async fn create(&self, content: &Content) -> Result<ID>;
    async fn get(&self, id: &str) -> Result<Provider>;
    async fn list(&self) -> Result<Vec<Provider>>;
}

#[derive(Debug, Deserialize)]
pub struct Content {
    pub secret: String,
    pub redirect_url: String,
    pub name: String,
    pub prompt: String,
    pub logo_url: String,
}
