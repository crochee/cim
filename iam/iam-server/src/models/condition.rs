use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use cim_core::{Error, Result};

use crate::services::req::Request;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct JsonCondition {
    #[serde(rename = "type")]
    pub jtype: String,
    pub options: Vec<u8>,
}

pub trait Condition {
    fn name(&self) -> String;
    fn fulfills(&self, req: &Request) -> bool;
}

impl Condition for JsonCondition {
    fn name(&self) -> String {
        "json".to_string()
    }

    fn fulfills(&self, req: &Request) -> bool {
        // let t: Condition =
        //     serde_json::from_slice(&self.options).map_err(Error::any)?;
        false
    }
}

impl JsonCondition {
    pub fn into_condition<T: Condition>(&self) -> Result<Vec<T>> {
        serde_json::from_slice(&self.options).map_err(Error::any)?;

        Ok(Vec::new())
    }
}

// lazy_static::lazy_static! {
//     static ref CONDITION_FACTORIES:HashMap<String,dyn FnOnce()->dyn Condition>=HashMap::from([
//         ("Mercury", 0.4),
//         ("Venus", 0.7),
//         ("Earth", 1.0),
//         ("Mars", 1.5),
//     ]);
// }
