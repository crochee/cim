mod im;

use std::sync::Arc;

use async_trait::async_trait;

use cim_core::Result;

use crate::{
    models::{usergroup::UserGroup, List, ID},
    repo::usergroups::{Content, Querys},
};

pub use im::IAMUserGroups;

pub type DynUserGroupsService = Arc<dyn UserGroupsService + Send + Sync>;

#[async_trait]
pub trait UserGroupsService {
    async fn create(&self, content: &Content) -> Result<ID>;
    async fn put(&self, id: &str, content: &Content) -> Result<()>;
    async fn get(
        &self,
        id: &str,
        account_id: Option<String>,
    ) -> Result<UserGroup>;
    async fn delete(&self, id: &str, account_id: Option<String>) -> Result<()>;
    async fn list(&self, filter: &Querys) -> Result<List<UserGroup>>;
}
