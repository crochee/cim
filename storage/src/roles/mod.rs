mod mariadb;

use async_trait::async_trait;
use chrono::NaiveDateTime;
use mockall::automock;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

pub use mariadb::RoleImpl;

use slo::Result;

use crate::{List, Pagination, ID};

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
pub struct Role {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub desc: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct Content {
    #[serde(skip)]
    pub account_id: String,
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[validate(length(min = 1, max = 255))]
    pub desc: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateOpts {
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,
    #[validate(length(min = 1, max = 255))]
    pub desc: Option<String>,
    #[serde(skip)]
    pub unscoped: Option<bool>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ListOpts {
    #[validate(length(min = 1))]
    pub account_id: Option<String>,
    #[validate(length(min = 1))]
    pub user_id: Option<String>,
    #[serde(flatten)]
    #[validate]
    pub pagination: Pagination,
}

#[derive(Debug, Deserialize, ToSchema)]
pub enum UserType {
    User = 1,
    Application = 2,
    FederatedUser = 3,
}

#[automock]
#[async_trait]
pub trait RoleStore {
    // roles
    async fn create_role(
        &self,
        id: Option<String>,
        content: &Content,
    ) -> Result<ID>;
    async fn update_role(
        &self,
        id: &str,
        account_id: Option<String>,
        opts: &UpdateOpts,
    ) -> Result<()>;
    async fn get_role(
        &self,
        id: &str,
        account_id: Option<String>,
    ) -> Result<Role>;
    async fn delete_role(
        &self,
        id: &str,
        account_id: Option<String>,
    ) -> Result<()>;
    async fn list_role(&self, list_opts: &ListOpts) -> Result<List<Role>>;
    async fn role_exist(
        &self,
        id: &str,
        account_id: Option<String>,
        unscoped: bool,
    ) -> Result<bool>;
    async fn attach_user(
        &self,
        id: &str,
        account_id: Option<String>,
        user_id: &str,
        user_type: UserType,
    ) -> Result<()>;
    async fn detach_user(
        &self,
        id: &str,
        user_id: &str,
        user_type: UserType,
    ) -> Result<()>;
}
