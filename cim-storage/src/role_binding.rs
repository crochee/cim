use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::Pagination;

#[derive(
    Debug, Default, Deserialize, Serialize, ToSchema, PartialEq, Clone,
)]
pub struct RoleBinding {
    pub id: String,
    pub role_id: String,
    pub user_type: UserType,
    pub user_id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct Content {
    #[validate(length(min = 1, max = 255))]
    pub role_id: String,
    pub user_type: UserType,
    #[validate(length(min = 1, max = 255))]
    pub user_id: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ListParams {
    #[validate(length(min = 1))]
    pub id: Option<String>,
    #[validate(length(min = 1))]
    pub role_id: Option<String>,
    pub user_type: Option<UserType>,
    #[validate(length(min = 1))]
    pub user_id: Option<String>,
    #[serde(flatten)]
    #[validate(nested)]
    pub pagination: Pagination,
}

#[derive(
    Debug, Default, Deserialize, Serialize, ToSchema, Clone, PartialEq,
)]
pub enum UserType {
    #[default]
    User = 1,
    Application = 2,
    FederatedUser = 3,
}

impl From<&UserType> for u8 {
    fn from(user_type: &UserType) -> Self {
        match user_type {
            UserType::User => 1,
            UserType::Application => 2,
            UserType::FederatedUser => 3,
        }
    }
}

impl From<u8> for UserType {
    fn from(user_type: u8) -> Self {
        match user_type {
            1 => UserType::User,
            2 => UserType::Application,
            3 => UserType::FederatedUser,
            _ => UserType::User,
        }
    }
}
