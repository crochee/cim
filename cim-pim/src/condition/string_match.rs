use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

use super::Condition;
use crate::req::Request;

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
