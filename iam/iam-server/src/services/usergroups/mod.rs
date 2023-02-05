mod im;

use std::sync::Arc;

use async_trait::async_trait;

use cim_core::Result;

use crate::{
    models::{
        usergroup::{UserGroup, UserGroupBindings},
        List, ID,
    },
    repo::usergroups::{Content, Querys},
};

pub use im::IAMUserGroups;

pub type DynUserGroups = Arc<dyn UserGroupsService + Send + Sync>;

#[async_trait]
pub trait UserGroupsService {
    async fn create(&self, content: &Content) -> Result<ID>;
    async fn put(&self, id: &str, content: &Content) -> Result<()>;
    async fn get(&self, id: &str, filter: &Querys)
        -> Result<UserGroupBindings>;
    async fn delete(&self, id: &str, account_id: Option<String>) -> Result<()>;
    async fn list(&self, filter: &Querys) -> Result<List<UserGroup>>;
    async fn add_user(
        &self,
        id: &str,
        account_id: &str,
        user_id: &str,
    ) -> Result<()>;
    async fn delete_user(&self, id: &str, user_id: &str) -> Result<()>;
    async fn add_role(
        &self,
        id: &str,
        account_id: &str,
        role_id: &str,
    ) -> Result<()>;
    async fn delete_role(&self, id: &str, role_id: &str) -> Result<()>;
}
