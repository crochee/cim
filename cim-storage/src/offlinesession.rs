use std::collections::HashMap;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use validator::Validate;

use crate::Pagination;

#[derive(Debug, Default, Deserialize, Serialize, Clone, utoipa::ToSchema)]
pub struct OfflineSession {
    pub id: String,
    pub user_id: String,
    pub conn_id: String,
    pub refresh: HashMap<String, RefreshTokenRef>,

    #[schema(format = Binary, value_type = String)]
    pub connector_data: Option<Box<RawValue>>,
}

impl PartialEq for OfflineSession {
    fn eq(&self, other: &Self) -> bool {
        if self.id == other.id
            && self.user_id == other.user_id
            && self.conn_id == other.conn_id
            && self.refresh == other.refresh
        {
            if let Some(connector_data) = &self.connector_data {
                if let Some(other_connector_data) = &other.connector_data {
                    return connector_data.get() == other_connector_data.get();
                }
            }
        }
        false
    }
}

#[derive(
    Debug, Default, Deserialize, Serialize, PartialEq, Clone, utoipa::ToSchema,
)]
pub struct RefreshTokenRef {
    pub id: String,
    pub client_id: String,
    pub created_at: NaiveDateTime,
    pub last_used_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ListParams {
    #[validate(length(min = 1))]
    pub user_id: Option<String>,
    #[validate(length(min = 1))]
    pub conn_id: Option<String>,
    #[serde(flatten)]
    #[validate(nested)]
    pub pagination: Pagination,
}
