mod mariadb;

use serde::Deserialize;
use validator::Validate;

use crate::models::{role::Role, Pagination};

pub use mariadb::*;

#[derive(Debug, Deserialize, Validate)]
pub struct Content {
    #[serde(skip)]
    pub account_id: String,
    #[serde(skip)]
    pub user_id: String,
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[validate(length(min = 1, max = 255))]
    pub desc: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct Opts {
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,
    #[validate(length(min = 1, max = 255))]
    pub desc: Option<String>,
    pub unscoped: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct Querys {
    #[validate(length(min = 1))]
    pub account_id: Option<String>,
    #[serde(flatten)]
    #[validate]
    pub pagination: Pagination,
}
