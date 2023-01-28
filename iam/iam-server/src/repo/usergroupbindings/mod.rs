mod mariadb;

use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use serde::Deserialize;
use validator::Validate;

use cim_core::Result;

use crate::models::{
    usergroupbinding::{Kind, UserGroupBinding},
    List, Pagination,
};

pub use mariadb::MariadbUserGroupBindings;

pub type DynUserGroupBindingsRepository =
    Arc<dyn UserGroupBindingsRepository + Send + Sync>;

#[automock]
#[async_trait]
pub trait UserGroupBindingsRepository {
    async fn create(&self, acount_id: String, content: &Content) -> Result<()>;

    async fn delete(&self, opts: &Opts) -> Result<()>;

    async fn list(&self, filter: &Querys) -> Result<List<UserGroupBinding>>;
}

#[derive(Debug, Deserialize, Validate)]
pub struct Content {
    pub items: Vec<UserGroupBinding>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct Opts {
    pub items: Vec<Opt>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct Opt {
    pub id: String,
    pub kind: Kind,
}

#[derive(Debug, Deserialize, Validate)]
pub struct Querys {
    #[validate(length(min = 1))]
    pub user_group_id: Option<String>,
    pub kind: Option<Kind>,
    #[serde(flatten)]
    #[validate]
    pub pagination: Pagination,
}
