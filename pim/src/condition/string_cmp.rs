use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

use super::Condition;
use crate::req::Request;

#[derive(Debug, Deserialize, Serialize)]
pub struct StringCmp {
    pub values: Vec<StringCmpInner>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StringCmpInner {
    pub equal: bool,
    pub ignore_case: bool,
    pub value: String,
}

impl Condition for StringCmp {
    fn evaluate(&self, input: Box<RawValue>, _req: &Request) -> bool {
        if let Ok(v) = serde_json::from_str::<String>(input.get()) {
            let mut matched = false;
            for dest_value in self.values.iter() {
                if !self.cmp(&v, dest_value) {
                    return false;
                }
                matched = true;
            }
            return matched;
        }
        false
    }
}

impl StringCmp {
    fn cmp(&self, src_value: &str, dest_value: &StringCmpInner) -> bool {
        if dest_value.ignore_case {
            let (src, dest) =
                (src_value.to_lowercase(), dest_value.value.to_lowercase());
            if dest_value.equal {
                return src == dest;
            }
            return src != dest;
        }
        if dest_value.equal {
            return src_value == dest_value.value;
        }
        src_value != dest_value.value
    }
}
