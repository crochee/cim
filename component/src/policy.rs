use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::statement::Statement;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct Policy {
    pub id: String,
    pub account_id: Option<String>,
    pub user_id: Option<String>,
    #[validate(length(min = 1))]
    pub desc: String,
    // 指定要使用的策略语言版本
    pub version: String,
    pub statement: Vec<Statement>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
