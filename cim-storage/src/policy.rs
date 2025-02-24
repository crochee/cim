use async_trait::async_trait;
use chrono::NaiveDateTime;
use mockall::automock;
use serde::{Deserialize, Serialize};
use validator::Validate;

use cim_pim::{Request, Statement};
use cim_slo::Result;

use crate::Pagination;

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Clone, utoipa::ToSchema)]
pub struct Policy {
    pub id: String,
    pub account_id: Option<String>,
    pub desc: String,
    // 指定要使用的策略语言版本
    pub version: String,
    pub statement: Vec<Statement>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, Validate)]
pub struct Content {
    #[validate(length(min = 1))]
    pub account_id: Option<String>,
    #[validate(length(min = 1, max = 255))]
    pub desc: String,
    // 指定要使用的策略语言版本
    #[validate(length(min = 1, max = 255))]
    pub version: String,
    #[validate(nested)]
    pub statement: Vec<Statement>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ListParams {
    #[validate(length(min = 1))]
    pub id: Option<String>,
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
    #[validate(nested)]
    pub pagination: Pagination,
}

#[automock]
#[async_trait]
pub trait StatementStore {
    async fn get_statement(&self, req: &Request) -> Result<Vec<Statement>>;
}
