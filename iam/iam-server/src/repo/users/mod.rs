mod mariadb;

use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use serde::Deserialize;
use validator::Validate;

use cim_core::Result;

use crate::{
    models::{user::User, List, Pagination, ID},
    pkg::valid::field::{check_password, check_sex},
};

pub use mariadb::MariadbUsers;

pub type DynUsersRepository = Arc<dyn UsersRepository + Send + Sync>;

#[automock]
#[async_trait]
pub trait UsersRepository {
    async fn create(&self, id: Option<String>, content: &Content)
        -> Result<ID>;

    async fn update(
        &self,
        id: &str,
        account_id: Option<String>,
        opts: &Opts,
    ) -> Result<()>;

    async fn get(&self, id: &str, account_id: Option<String>) -> Result<User>;

    async fn delete(&self, id: &str, account_id: Option<String>) -> Result<()>;

    async fn list(&self, filter: &Querys) -> Result<List<User>>;

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

#[derive(Debug, Deserialize, Validate)]
pub struct Opts {
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
    pub unscoped: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct Querys {
    #[validate(length(min = 1))]
    pub account_id: Option<String>,
    #[validate(custom = "check_sex")]
    pub sex: Option<String>,
    #[serde(flatten)]
    #[validate]
    pub pagination: Pagination,
}
