use std::net::IpAddr;

use chrono::prelude::*;
use cidr_utils::cidr::IpCidr;
use regex::Regex;
use serde::{Deserialize, Serialize};

use cim_core::{Code, Result};
use serde_json::value::RawValue;

use super::req::Request;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct JsonCondition {
    #[serde(rename = "type")]
    pub jtype: String,
    pub options: Box<RawValue>,
}

impl JsonCondition {
    pub fn into(&self) -> Result<Box<dyn Condition>> {
        match self.jtype.as_str() {
            "StringCmp" => {
                let result: StringCmp =
                    serde_json::from_str(self.options.get())
                        .map_err(Code::any)?;
                Ok(Box::new(result))
            }
            "StringMatch" => {
                let result: StringMatch =
                    serde_json::from_str(self.options.get())
                        .map_err(Code::any)?;
                Ok(Box::new(result))
            }
            "CIDR" => {
                let result: Cidr = serde_json::from_str(self.options.get())
                    .map_err(Code::any)?;
                Ok(Box::new(result))
            }
            "EqualsSubject" => {
                let result: EqualsSubject =
                    serde_json::from_str(self.options.get())
                        .map_err(Code::any)?;
                Ok(Box::new(result))
            }
            "Boolean" => {
                let result: Boolean = serde_json::from_str(self.options.get())
                    .map_err(Code::any)?;
                Ok(Box::new(result))
            }
            "NumericCmp" => {
                let result: NumericCmp =
                    serde_json::from_str(self.options.get())
                        .map_err(Code::any)?;
                Ok(Box::new(result))
            }
            "TimeCmp" => {
                let result: TimeCmp = serde_json::from_str(self.options.get())
                    .map_err(Code::any)?;
                Ok(Box::new(result))
            }
            v => Err(Code::not_found(&format!(
                "Could not find condition type {}",
                v
            ))),
        }
    }
}

pub trait Condition {
    fn evaluate(&self, input: Box<RawValue>, req: &Request) -> bool;
}

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

#[derive(Debug, Deserialize, Serialize)]
pub struct StringMatch {
    pub matches: String,
}

impl Condition for StringMatch {
    fn evaluate(&self, input: Box<RawValue>, _req: &Request) -> bool {
        if let Ok(v) = serde_json::from_str::<String>(input.get()) {
            if let Ok(matcher) = Regex::new(&self.matches) {
                return matcher.is_match(&v);
            }
        }
        false
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Cidr {
    pub cidr: Vec<String>,
}

impl Condition for Cidr {
    fn evaluate(&self, input: Box<RawValue>, _req: &Request) -> bool {
        if let Ok(v) = serde_json::from_str::<String>(input.get()) {
            let mut matched = false;
            for value in self.cidr.iter() {
                if let Ok(cidr) = IpCidr::from_str(value) {
                    if let Ok(ip) = v.parse::<IpAddr>() {
                        if !cidr.contains(ip) {
                            return false;
                        }
                        matched = true;
                    }
                }
            }
            return matched;
        }
        false
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EqualsSubject {}

impl Condition for EqualsSubject {
    fn evaluate(&self, input: Box<RawValue>, req: &Request) -> bool {
        if let Ok(v) = serde_json::from_str::<String>(input.get()) {
            return req.subject == v;
        }
        false
    }
}

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
                        if let Ok(src) = DateTime::parse_from_str(src_value, v)
                        {
                            if let Ok(dest) =
                                DateTime::parse_from_str(&dest_value.value, v)
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
        if let Ok(src) = DateTime::parse_from_str(src_value, format) {
            if let Ok(dest) =
                DateTime::parse_from_str(&dest_value.value, format)
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
