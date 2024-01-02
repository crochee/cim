pub(crate) mod boolean;
pub(crate) mod cidr;
pub(crate) mod numeric_cmp;
pub(crate) mod string_cmp;
pub(crate) mod string_match;
pub(crate) mod time_cmp;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

use crate::req::Request;

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
                let result: string_cmp::StringCmp =
                    serde_json::from_str(self.options.get())
                        .context("Could not parse StringCmp")?;
                Ok(Box::new(result))
            }
            "StringMatch" => {
                let result: string_match::StringMatch =
                    serde_json::from_str(self.options.get())
                        .context("Could not parse StringMatch")?;
                Ok(Box::new(result))
            }
            "CIDR" => {
                let result: cidr::Cidr =
                    serde_json::from_str(self.options.get())
                        .context("Could not parse Cidr")?;
                Ok(Box::new(result))
            }
            "Boolean" => {
                let result: boolean::Boolean =
                    serde_json::from_str(self.options.get())
                        .context("Could not parse Boolean")?;
                Ok(Box::new(result))
            }
            "NumericCmp" => {
                let result: numeric_cmp::NumericCmp =
                    serde_json::from_str(self.options.get())
                        .context("Could not parse NumericCmp")?;
                Ok(Box::new(result))
            }
            "TimeCmp" => {
                let result: time_cmp::TimeCmp =
                    serde_json::from_str(self.options.get())
                        .context("Could not parse TimeCmp")?;
                Ok(Box::new(result))
            }
            v => Err(anyhow::anyhow!("Could not find condition type {}", v)),
        }
    }
}

pub trait Condition {
    fn evaluate(&self, input: Box<RawValue>, req: &Request) -> bool;
}
