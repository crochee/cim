use std::collections::HashMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use validator::Validate;

use crate::condition::JsonCondition;
use crate::req::Request;

#[derive(Debug, Deserialize, Serialize, Validate, Clone)]
pub struct Statement {
    pub sid: Option<String>,
    pub effect: Effect,
    pub subjects: Vec<String>,
    pub actions: Vec<String>,
    pub resources: Vec<String>,
    pub conditions: Option<HashMap<String, JsonCondition>>,
    pub meta: Option<Box<RawValue>>,
}

impl Statement {
    pub fn get_start_delimiter(&self) -> char {
        '<'
    }

    pub fn get_end_delimiter(&self) -> char {
        '>'
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Effect {
    Allow,
    Deny,
}
