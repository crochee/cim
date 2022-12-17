mod mariadb;

use std::sync::Arc;

use async_trait::async_trait;

pub type DynPoliciesRepository = Arc<dyn PoliciesRepository + Send + Sync>;
use cim_core::Result;
pub use mariadb::MariadbPolicies;
use serde::Deserialize;
use validator::Validate;

use crate::models::{
    policy::{Policy, Statement},
    List, Pagination, ID,
};

#[async_trait]
pub trait PoliciesRepository {
    async fn create(&self, id: Option<String>, content: &Content)
        -> Result<ID>;

    async fn update(
        &self,
        id: &str,
        account_id: Option<String>,
        opts: &Opts,
    ) -> Result<()>;

    async fn get(&self, id: &str, account_id: Option<String>)
        -> Result<Policy>;

    async fn delete(&self, id: &str, account_id: Option<String>) -> Result<()>;

    async fn list(&self, filter: &Querys) -> Result<List<Policy>>;

    async fn exist(
        &self,
        id: &str,
        account_id: Option<String>,
        unscoped: bool,
    ) -> Result<bool>;
}

#[derive(Debug, Deserialize, Validate)]
pub struct Content {
    #[validate(length(min = 1))]
    pub account_id: Option<String>,
    #[validate(length(min = 1))]
    pub user_id: Option<String>,
    #[validate(length(min = 1))]
    pub desc: String,
    // 指定要使用的策略语言版本
    pub version: String,
    pub statement: Vec<Statement>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct Opts {
    #[validate(length(min = 1))]
    pub desc: Option<String>,
    #[validate(length(min = 1))]
    pub version: Option<String>,
    pub statement: Option<Vec<Statement>>,
    pub unscoped: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct Querys {
    #[validate(length(min = 1))]
    pub version: Option<String>,
    #[validate(length(min = 1))]
    pub account_id: Option<String>,
    #[serde(flatten)]
    #[validate]
    pub pagination: Pagination,
}
