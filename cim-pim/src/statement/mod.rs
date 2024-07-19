use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use validator::Validate;

use crate::condition::JsonCondition;

#[derive(Debug, Deserialize, Serialize, Validate, Clone)]
pub struct Statement {
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

impl PartialEq for Statement {
    fn eq(&self, other: &Self) -> bool {
        if self.effect == other.effect
            && self.subjects == other.subjects
            && self.actions == other.actions
            && self.resources == other.resources
            && self.conditions == other.conditions
        {
            if let (Some(meta1), Some(meta2)) = (&self.meta, &other.meta) {
                return meta1.get() == meta2.get();
            }
        }
        false
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub enum Effect {
    Allow,
    Deny,
}
