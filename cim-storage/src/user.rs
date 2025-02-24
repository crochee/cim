use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use validator::Validate;

use cim_slo::regexp::check_password;

use crate::{ClaimOpts, Pagination};

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Clone, utoipa::ToSchema)]
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

#[derive(Debug, Deserialize, Validate)]
pub struct Content {
    #[validate(length(min = 1))]
    pub account_id: Option<String>,
    pub desc: String,
    #[serde(flatten)]
    #[validate(nested)]
    pub claim: ClaimOpts,
    #[validate(custom(function = "check_password"))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ListParams {
    #[validate(length(min = 1))]
    pub id: Option<String>,
    #[validate(length(min = 1))]
    pub account_id: Option<String>,
    #[validate(length(min = 1))]
    pub group_id: Option<String>,
    #[serde(flatten)]
    #[validate(nested)]
    pub pagination: Pagination,
}
