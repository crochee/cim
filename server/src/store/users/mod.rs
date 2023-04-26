mod mariadb;

use serde::Deserialize;
use validator::Validate;

use crate::{
    models::Pagination,
    pkg::valid::field::{check_password, check_sex},
};

pub use mariadb::*;

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

#[derive(Debug)]
pub enum UserSubject {
    UserID(String),
    Email(String),
    Mobile(String),
}

#[derive(Debug)]
pub struct Password {
    pub user_id: String,
    pub user_name: String,
    pub nick_name: String,
    pub email: Option<String>,
    pub mobile: Option<String>,
    pub hash: String,
    pub secret: String,
}
