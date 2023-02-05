mod mariadb;

use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use serde::Deserialize;
use validator::Validate;

use cim_core::Result;

use crate::models::{
    role::{Role, RoleBindings},
    List, Pagination, ID,
};

pub use mariadb::MariadbRoles;

pub type DynRoles = Arc<dyn RolesRep + Send + Sync>;

#[automock]
#[async_trait]
pub trait RolesRep {
    async fn create(&self, id: Option<String>, content: &Content)
        -> Result<ID>;

    async fn update(
        &self,
        id: &str,
        account_id: Option<String>,
        opts: &Opts,
    ) -> Result<()>;

    async fn get(&self, id: &str, filter: &Querys) -> Result<RoleBindings>;

    async fn delete(&self, id: &str, account_id: Option<String>) -> Result<()>;

    async fn list(&self, filter: &Querys) -> Result<List<Role>>;

    async fn exist(
        &self,
        id: &str,
        account_id: Option<String>,
        unscoped: bool,
    ) -> Result<bool>;

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

#[derive(Debug, Deserialize, Validate)]
pub struct Content {
    #[serde(skip)]
    pub account_id: String,
    #[serde(skip)]
    pub user_id: String,
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[validate(length(min = 1, max = 255))]
    pub desc: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct Opts {
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,
    #[validate(length(min = 1, max = 255))]
    pub desc: Option<String>,
    pub unscoped: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct Querys {
    #[validate(length(min = 1))]
    pub account_id: Option<String>,
    #[serde(flatten)]
    #[validate]
    pub pagination: Pagination,
}
