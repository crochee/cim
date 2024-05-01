mod mariadb;
mod sql;

use async_trait::async_trait;
use chrono::NaiveDateTime;
use mockall::automock;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

pub use mariadb::UserImpl;

use cim_slo::{regexp::check_password, Result};

use crate::{ClaimOpts, Interface, List, Pagination, ID};

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct User {
    pub id: String,
    pub account_id: String,
    pub desc: String,
    #[serde(flatten)]
    pub claim: ClaimOpts,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct Content {
    #[validate(length(min = 1))]
    pub account_id: Option<String>,
    pub desc: String,
    #[serde(flatten)]
    pub claim: ClaimOpts,
    #[validate(custom = "check_password")]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateOpts {
    pub desc: Option<String>,
    #[serde(flatten)]
    pub claim: ClaimOpts,
    #[validate(custom = "check_password")]
    pub password: Option<String>,
    #[serde(skip)]
    pub unscoped: Option<bool>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ListOpts {
    #[validate(length(min = 1))]
    pub account_id: Option<String>,
    #[validate(length(min = 1))]
    pub group_id: Option<String>,
    #[serde(flatten)]
    #[validate]
    pub pagination: Pagination,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ListParams {
    #[validate(length(min = 1))]
    pub id: Option<String>,
    #[validate(length(min = 1))]
    pub account_id: Option<String>,
    #[validate(length(min = 1))]
    pub group_id: Option<String>,
}

pub struct CountOpts {
    pub id: Option<String>,
    pub account_id: Option<String>,
}

#[automock]
#[async_trait]
pub trait UserStore {
    // users
    async fn create_user(
        &self,
        id: Option<String>,
        content: &Content,
    ) -> Result<ID>;
    async fn update_user(
        &self,
        id: &str,
        account_id: Option<String>,
        opts: &UpdateOpts,
    ) -> Result<()>;
    async fn get_user(
        &self,
        id: &str,
        account_id: Option<String>,
    ) -> Result<User>;
    async fn get_user_password(&self, id: &str) -> Result<User>;
    async fn delete_user(
        &self,
        id: &str,
        account_id: Option<String>,
    ) -> Result<()>;
    async fn list_user(&self, list_opts: &ListOpts) -> Result<List<User>>;
    async fn user_exist(
        &self,
        id: &str,
        account_id: Option<String>,
        unscoped: bool,
    ) -> Result<bool>;
}

pub fn nick_name_generator(name: &str) -> String {
    format!("用户_{}", name)
}

#[async_trait]
pub trait UserInterface: Interface {}
