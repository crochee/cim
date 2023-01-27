mod im;

use std::sync::Arc;

use async_trait::async_trait;

use cim_core::Result;

use crate::{
    models::{role::Role, List, ID},
    repo::roles::{Content, Querys},
};

pub use im::IAMRoles;

pub type DynRolesService = Arc<dyn RolesService + Send + Sync>;

#[async_trait]
pub trait RolesService {
    async fn create(&self, content: &Content) -> Result<ID>;
    async fn put(&self, id: &str, content: &Content) -> Result<()>;
    async fn get(&self, id: &str, account_id: Option<String>) -> Result<Role>;
    async fn delete(&self, id: &str, account_id: Option<String>) -> Result<()>;
    async fn list(&self, filter: &Querys) -> Result<List<Role>>;
}
