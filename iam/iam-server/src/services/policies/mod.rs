mod im;

use std::sync::Arc;

use async_trait::async_trait;

use cim_core::Result;

pub type DynPoliciesService = Arc<dyn PoliciesService + Send + Sync>;
pub use im::IAMPolicies;

use crate::{
    models::{policy::Policy, List, ID},
    repo::policies::{Content, Querys},
};

#[async_trait]
pub trait PoliciesService {
    async fn create(&self, content: &Content) -> Result<ID>;
    async fn put(&self, id: &str, content: &Content) -> Result<()>;
    async fn get(&self, id: &str, account_id: Option<String>)
        -> Result<Policy>;
    async fn delete(&self, id: &str, account_id: Option<String>) -> Result<()>;
    async fn list(&self, filter: &Querys) -> Result<List<Policy>>;
}
