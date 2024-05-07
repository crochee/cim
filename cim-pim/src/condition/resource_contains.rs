use std::collections::HashMap;

use serde_json::value::RawValue;

use super::Condition;
use crate::req::Request;

#[derive(Debug)]
pub struct ResourceContains;

impl Condition for ResourceContains {
    fn evaluate(&self, input: Box<RawValue>, req: &Request) -> bool {
        if let Ok(v) =
            serde_json::from_str::<HashMap<String, String>>(input.get())
        {
            let value = match v.get("value") {
                Some(value_string) => {
                    if value_string.is_empty() {
                        return false;
                    }
                    value_string.to_string()
                }
                None => return false,
            };
            let delimiter = v
                .get("delimiter")
                .map(|v| v.to_string())
                .unwrap_or("".to_string());
            let mut filter_value = String::from("");
            filter_value.push_str(&delimiter);
            filter_value.push_str(&value);
            filter_value.push_str(&delimiter);

            let mut resource_value = String::from("");
            resource_value.push_str(&delimiter);
            resource_value.push_str(&req.resource);
            resource_value.push_str(&delimiter);

            return resource_value.contains(&filter_value);
        }
        false
    }
}
