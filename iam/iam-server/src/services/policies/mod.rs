mod condition;
mod im;

use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use cim_core::Result;
use mockall::automock;
use serde::{Deserialize, Serialize};

pub type DynPoliciesService = Arc<dyn PoliciesService + Send + Sync>;
pub use im::IAMPolicies;
use validator::Validate;

use crate::models::{List, Pagination, ID};

#[derive(Debug, Deserialize, Serialize)]
pub struct Policy {
    pub id: String,
    pub description: String,
    pub subjects: Vec<String>,
    pub effect: Effect,
    pub resources: Vec<String>,
    pub actions: Vec<String>,
    pub collections: HashMap<String, condition::JsonCondition>,
    pub meta: Vec<u8>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Effect {
    Allow,
    Deny,
}

#[derive(Debug, Deserialize, Validate)]
pub struct Filter {
    #[validate(length(min = 1))]
    pub name: Option<String>,
    #[validate(length(min = 1))]
    pub account_id: Option<String>,
    #[serde(flatten)]
    #[validate]
    pub pagination: Pagination,
}

#[automock]
#[async_trait]
pub trait PoliciesService {
    async fn create(&self, policy: &Policy) -> Result<ID>;
    async fn put(&self, id: &str, policy: &Policy) -> Result<()>;
    async fn get(&self, id: &str) -> Result<Policy>;
    async fn delete(&self, id: &str) -> Result<()>;
    async fn list(&self, filter: &Filter) -> Result<List<Policy>>;
    async fn exist(&self, id: &str) -> Result<()>;
}
