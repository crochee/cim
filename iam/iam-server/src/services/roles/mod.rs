mod im;

use std::sync::Arc;

use async_trait::async_trait;

use cim_core::Result;

use crate::{
    models::{
        role::{Role, RoleBindings},
        List, ID,
    },
    repo::roles::{Content, Querys},
};

pub use im::IAMRoles;

pub type DynRoles = Arc<dyn RolesService + Send + Sync>;

#[async_trait]
pub trait RolesService {
    async fn create(&self, content: &Content) -> Result<ID>;
    async fn put(&self, id: &str, content: &Content) -> Result<()>;
    async fn get(&self, id: &str, filter: &Querys) -> Result<RoleBindings>;
    async fn delete(&self, id: &str, account_id: Option<String>) -> Result<()>;
    async fn list(&self, filter: &Querys) -> Result<List<Role>>;
    async fn add_user(
        &self,
        id: &str,
        account_id: &str,
        user_id: &str,
    ) -> Result<()>;
    async fn delete_user(&self, id: &str, user_id: &str) -> Result<()>;
    async fn add_policy(
        &self,
        id: &str,
        account_id: &str,
        policy_id: &str,
    ) -> Result<()>;
    async fn delete_policy(&self, id: &str, policy_id: &str) -> Result<()>;
}
