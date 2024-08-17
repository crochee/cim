use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use utoipa::ToSchema;
use validator::Validate;

use crate::Pagination;

#[derive(Debug, Default, Deserialize, Serialize, ToSchema, Clone)]
pub struct Connector {
    pub id: String,
    #[serde(rename = "type")]
    pub connector_type: String,
    pub name: String,
    pub response_version: String,
    pub config: String,
    pub connector_data: Option<Box<RawValue>>,
}

impl PartialEq for Connector {
    fn eq(&self, other: &Self) -> bool {
        if self.id == other.id
            && self.connector_type == other.connector_type
            && self.name == other.name
            && self.response_version == other.response_version
            && self.config == other.config
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

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct ListParams {
    #[validate(length(min = 1))]
    #[serde(rename = "type")]
    pub connector_type: Option<String>,
    #[serde(flatten)]
    #[validate(nested)]
    pub pagination: Pagination,
}
