pub mod mariadb;

use async_trait::async_trait;
use chrono::NaiveDateTime;
use mockall::automock;
use pim::{Request, Statement};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use slo::Result;

use crate::{List, Pagination, ID};

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
pub struct Policy {
    pub id: String,
    pub account_id: Option<String>,
    #[validate(length(min = 1))]
    pub desc: String,
    // 指定要使用的策略语言版本
    pub version: String,
    pub statement: Vec<Statement>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct Content {
    #[validate(length(min = 1))]
    pub account_id: Option<String>,
    #[validate(length(min = 1, max = 255))]
    pub desc: String,
    // 指定要使用的策略语言版本
    #[validate(length(min = 1, max = 255))]
    pub version: String,
    #[validate]
    pub statement: Vec<Statement>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateOpts {
    #[validate(length(min = 1))]
    pub desc: Option<String>,
    #[validate(length(min = 1))]
    pub version: Option<String>,
    pub statement: Option<Vec<Statement>>,
    #[serde(skip)]
    pub unscoped: Option<bool>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ListOpts {
    #[validate(length(min = 1))]
    pub version: Option<String>,
    #[validate(length(min = 1))]
    pub account_id: Option<String>,
    #[validate(length(min = 1))]
    pub group_id: Option<String>,
    #[validate(length(min = 1))]
    pub user_id: Option<String>,
    #[validate(length(min = 1))]
    pub role_id: Option<String>,
    #[serde(flatten)]
    #[validate]
    pub pagination: Pagination,
}

#[derive(Debug, Deserialize, ToSchema)]
pub enum BindingsType {
    User = 1,
    Group = 2,
    Role = 3,
}

#[automock]
#[async_trait]
pub trait PolicyStore {
    async fn create_policy(
        &self,
        id: Option<String>,
        content: &Content,
    ) -> Result<ID>;
    async fn update_policy(
        &self,
        id: &str,
        account_id: Option<String>,
        opts: &UpdateOpts,
    ) -> Result<()>;
    async fn get_policy(
        &self,
        id: &str,
        account_id: Option<String>,
    ) -> Result<Policy>;
    async fn delete_policy(
        &self,
        id: &str,
        account_id: Option<String>,
    ) -> Result<()>;
    async fn list_policy(&self, list_opts: &ListOpts) -> Result<List<Policy>>;
    async fn policy_exist(
        &self,
        id: &str,
        account_id: Option<String>,
        unscoped: bool,
    ) -> Result<bool>;

    async fn attach(
        &self,
        id: &str,
        account_id: Option<String>,
        bindings_id: &str,
        bindings_type: BindingsType,
    ) -> Result<()>;
    async fn detach(
        &self,
        id: &str,
        bindings_id: &str,
        bindings_type: BindingsType,
    ) -> Result<()>;
}

#[automock]
#[async_trait]
pub trait StatementStore {
    async fn get_statement(&self, req: &Request) -> Result<Vec<Statement>>;
}
