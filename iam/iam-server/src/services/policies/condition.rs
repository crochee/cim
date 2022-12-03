use serde::{Deserialize, Serialize};

use cim_core::Result;

use crate::services::req::Request;

#[derive(Debug, Deserialize, Serialize)]
pub struct JsonCondition {
    #[serde(rename = "type")]
    pub jtype: String,
    pub options: Vec<u8>,
}

pub trait Condition{
    fn name(&self) -> String;
    fn fulfills<T>(&self, req: &Request) -> bool;
}

impl JsonCondition {
    pub fn into_condition<T: Condition>(&self) -> Result<Vec<T>>{
        

        Ok(Vec::new())

    }
}