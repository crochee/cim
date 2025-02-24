use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::Pagination;

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Clone, utoipa::ToSchema)]
pub struct GroupUser {
    pub id: String,
    pub group_id: String,
    pub user_id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, Validate)]
pub struct Content {
    #[validate(length(min = 1, max = 255))]
    pub group_id: String,
    #[validate(length(min = 1, max = 255))]
    pub user_id: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ListParams {
    #[validate(length(min = 1))]
    pub id: Option<String>,
    #[validate(length(min = 1))]
    pub group_id: Option<String>,
    #[validate(length(min = 1))]
    pub user_id: Option<String>,
    #[serde(flatten)]
    #[validate(nested)]
    pub pagination: Pagination,
}
