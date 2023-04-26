mod mariadb;

use serde::Deserialize;
use validator::Validate;

use crate::models::{policy::Statement, Pagination};

pub use mariadb::*;

#[derive(Debug, Deserialize, Validate)]
pub struct Content {
    #[validate(length(min = 1))]
    pub account_id: Option<String>,
    #[validate(length(min = 1))]
    pub user_id: Option<String>,
    #[validate(length(min = 1, max = 255))]
    pub desc: String,
    // 指定要使用的策略语言版本
    #[validate(length(min = 1, max = 255))]
    pub version: String,
    #[validate]
    pub statement: Vec<Statement>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct Opts {
    #[validate(length(min = 1))]
    pub desc: Option<String>,
    #[validate(length(min = 1))]
    pub version: Option<String>,
    pub statement: Option<Vec<Statement>>,
    #[serde(skip)]
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
