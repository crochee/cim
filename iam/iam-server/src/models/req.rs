use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct Req {
    pub resource: String,
    pub action: String,
    pub subject: String,
    pub context: HashMap<String, Vec<u8>>,
}
