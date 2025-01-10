use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::Pagination;

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Clone)]
pub struct PolicyBinding {
    pub id: String,
    pub policy_id: String,
    pub bindings_type: BindingsType,
    pub bindings_id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, Validate)]
pub struct Content {
    #[validate(length(min = 1, max = 255))]
    pub policy_id: String,
    pub bindings_type: BindingsType,
    #[validate(length(min = 1, max = 255))]
    pub bindings_id: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ListParams {
    #[validate(length(min = 1))]
    pub id: Option<String>,
    #[validate(length(min = 1))]
    pub policy_id: Option<String>,
    pub bindings_type: Option<BindingsType>,
    #[validate(length(min = 1))]
    pub bindings_id: Option<String>,
    #[serde(flatten)]
    #[validate(nested)]
    pub pagination: Pagination,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
pub enum BindingsType {
    #[default]
    User = 1,
    Group = 2,
    Role = 3,
}

impl From<&BindingsType> for u8 {
    fn from(bindings_type: &BindingsType) -> Self {
        match bindings_type {
            BindingsType::User => 1,
            BindingsType::Group => 2,
            BindingsType::Role => 3,
        }
    }
}

impl From<u8> for BindingsType {
    fn from(bindings_type: u8) -> Self {
        match bindings_type {
            1 => BindingsType::User,
            2 => BindingsType::Group,
            3 => BindingsType::Role,
            _ => BindingsType::User,
        }
    }
}
