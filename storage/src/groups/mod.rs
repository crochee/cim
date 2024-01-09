pub mod mariadb;

use async_trait::async_trait;
use chrono::NaiveDateTime;
use mockall::automock;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use slo::Result;

use crate::{List, Pagination, ID};

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
pub struct Group {
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
    #[serde(flatten)]
    #[validate]
    pub pagination: Pagination,
}

#[automock]
#[async_trait]
pub trait GroupStore {
    async fn create_group(
        &self,
        id: Option<String>,
        content: &Content,
    ) -> Result<ID>;
    async fn update_group(
        &self,
        id: &str,
        account_id: Option<String>,
        opts: &UpdateOpts,
    ) -> Result<()>;
    async fn get_group(
        &self,
        id: &str,
        account_id: Option<String>,
    ) -> Result<Group>;
    async fn delete_group(
        &self,
        id: &str,
        account_id: Option<String>,
    ) -> Result<()>;
    async fn list_group(&self, list_opts: &ListOpts) -> Result<List<Group>>;
    async fn group_exist(
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
    ) -> Result<()>;
    async fn detach_user(&self, id: &str, user_id: &str) -> Result<()>;
}
