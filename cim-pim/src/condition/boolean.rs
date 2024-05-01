use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

use super::Condition;
use crate::req::Request;

#[derive(Debug, Deserialize, Serialize)]
pub struct Boolean {
    pub value: bool,
}

impl Condition for Boolean {
    fn evaluate(&self, input: Box<RawValue>, _req: &Request) -> bool {
        if let Ok(v) = serde_json::from_str::<bool>(input.get()) {
            return self.value == v;
        }
        false
    }
}
