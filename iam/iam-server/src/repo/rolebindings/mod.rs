mod mariadb;

use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use serde::Deserialize;
use validator::Validate;

use cim_core::Result;

use crate::models::{
    rolebinding::{Kind, RoleBinding},
    List, Pagination,
};

pub use mariadb::MariadbRoleBindings;

pub type DynRoleBindingsRepository =
    Arc<dyn RoleBindingsRepository + Send + Sync>;

#[automock]
#[async_trait]
pub trait RoleBindingsRepository {
    async fn create(&self, acount_id: String, content: &Content) -> Result<()>;

    async fn delete(&self, opts: &Opts) -> Result<()>;

    async fn list(&self, filter: &Querys) -> Result<List<RoleBinding>>;
}

#[derive(Debug, Deserialize, Validate)]
pub struct Content {
    pub items: Vec<RoleBinding>,
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
    pub role_id: Option<String>,
    pub kind: Option<Kind>,
    #[serde(flatten)]
    #[validate]
    pub pagination: Pagination,
}
