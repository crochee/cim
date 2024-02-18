pub mod mariadb;

use async_trait::async_trait;
use chrono::NaiveDateTime;
use mockall::automock;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use slo::{
    regexp::{check_password, check_sex},
    Result,
};

use crate::{List, Pagination, ID};

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct User {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub nick_name: String,
    pub desc: String,
    pub email: Option<String>,
    pub mobile: Option<String>,
    pub sex: Option<String>,
    pub image: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct Content {
    #[validate(length(min = 1))]
    pub account_id: Option<String>,
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[validate(length(min = 1, max = 255))]
    pub nick_name: Option<String>,
    #[validate(length(min = 1, max = 255))]
    pub desc: String,
    #[validate(email)]
    pub email: Option<String>,
    #[validate(length(equal = 11))]
    pub mobile: Option<String>,
    #[validate(custom = "check_sex")]
    pub sex: Option<String>,
    #[validate(url)]
    pub image: Option<String>,
    #[validate(custom = "check_password")]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateOpts {
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,
    #[validate(length(min = 1, max = 255))]
    pub nick_name: Option<String>,
    #[validate(length(min = 1, max = 255))]
    pub desc: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    #[validate(length(equal = 11))]
    pub mobile: Option<String>,
    #[validate(custom = "check_sex")]
    pub sex: Option<String>,
    #[validate(url)]
    pub image: Option<String>,
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
    #[validate(custom = "check_sex")]
    pub sex: Option<String>,
    #[serde(flatten)]
    #[validate]
    pub pagination: Pagination,
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
