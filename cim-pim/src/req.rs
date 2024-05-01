use std::collections::HashMap;

use serde::Deserialize;
use serde_json::value::RawValue;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, Clone)]
pub struct Request {
    pub resource: String,
    pub action: String,
    pub subject: String,
    pub context: HashMap<String, Box<RawValue>>,
}
