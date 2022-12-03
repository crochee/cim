use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct Request {
    pub resource: String,
    pub action: String,
    pub subject: String,
    pub env: HashMap<String, Vec<u8>>,
}

#[derive(Debug, Validate)]
pub struct Attributes {
    // subject item
    #[validate(length(min = 1))]
    pub account_id: String,
    #[validate(length(min = 1))]
    pub user_id: String,
    #[validate(length(min = 1))]
    pub name: String,
    pub role_id: String,
    #[validate(length(min = 1))]
    pub action: String,
    // object item
    #[validate(length(min = 1))]
    pub object_id: String,
    #[validate(length(min = 1))]
    pub object_name: String,
    pub env: HashMap<String, HashMap<String, Vec<String>>>,
}
