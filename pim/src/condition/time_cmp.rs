use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

use super::Condition;
use crate::req::Request;

#[derive(Debug, Deserialize, Serialize)]
pub struct TimeCmp {
    pub values: Vec<TimeCmpInner>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TimeCmpInner {
    pub symbol: String,
    pub value: String,
    pub format: String,
    pub location: Option<String>,
}

impl Condition for TimeCmp {
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

impl TimeCmp {
    fn cmp(&self, src_value: &str, dest_value: &TimeCmpInner) -> bool {
        match dest_value.format.as_str() {
            "unix" | "unixnano" => {
                if let Ok(src) = src_value.parse::<i64>() {
                    if let Ok(dest) = dest_value.value.parse::<i64>() {
                        match dest_value.symbol.as_str() {
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
            v => match &dest_value.location {
                Some(location) => match location.as_str() {
                    "UTC" => {
                        return self.cmp_utc(src_value, dest_value, v);
                    }
                    "LOCAL" => {
                        if let Ok(src) =
                            NaiveDateTime::parse_from_str(src_value, v)
                        {
                            if let Ok(dest) = NaiveDateTime::parse_from_str(
                                &dest_value.value,
                                v,
                            ) {
                                match dest_value.symbol.as_str() {
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
                    _ => {}
                },
                None => {
                    return self.cmp_utc(src_value, dest_value, v);
                }
            },
        }
        false
    }

    fn cmp_utc(
        &self,
        src_value: &str,
        dest_value: &TimeCmpInner,
        format: &str,
    ) -> bool {
        if let Ok(src) = NaiveDateTime::parse_from_str(src_value, format) {
            if let Ok(dest) =
                NaiveDateTime::parse_from_str(&dest_value.value, format)
            {
                match dest_value.symbol.as_str() {
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
        false
    }
}
