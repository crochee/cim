use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

use super::Condition;
use crate::req::Request;

#[derive(Debug, Deserialize, Serialize)]
pub struct NumericCmp {
    pub symbol: String,
    pub value: serde_json::Number,
}

impl Condition for NumericCmp {
    fn evaluate(&self, input: Box<RawValue>, _req: &Request) -> bool {
        if let Ok(v) = serde_json::from_str::<serde_json::Number>(input.get()) {
            if v.is_f64() {
                if let Some(src) = v.as_f64() {
                    if let Some(dest) = self.value.as_f64() {
                        match self.symbol.as_str() {
                            "==" => return src == dest,
                            "!=" => return src != dest,
                            ">" => return src > dest,
                            ">=" => return src >= dest,
                            "<" => return src < dest,
                            "<=" => return src <= dest,
                            _ => {}
                        }
                    }
                }
            } else if v.is_i64() {
                if let Some(src) = v.as_i64() {
                    if let Some(dest) = self.value.as_i64() {
                        match self.symbol.as_str() {
                            "==" => return src == dest,
                            "!=" => return src != dest,
                            ">" => return src > dest,
                            ">=" => return src >= dest,
                            "<" => return src < dest,
                            "<=" => return src <= dest,
                            _ => {}
                        }
                    }
                }
            } else if v.is_u64() {
                if let Some(src) = v.as_u64() {
                    if let Some(dest) = self.value.as_u64() {
                        match self.symbol.as_str() {
                            "==" => return src == dest,
                            "!=" => return src != dest,
                            ">" => return src > dest,
                            ">=" => return src >= dest,
                            "<" => return src < dest,
                            "<=" => return src <= dest,
                            _ => {}
                        }
                    }
                }
            }
        }
        false
    }
}
