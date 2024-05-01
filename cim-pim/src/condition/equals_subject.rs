use serde_json::value::RawValue;

use super::Condition;
use crate::req::Request;

#[derive(Debug)]
pub struct EqualsSubject;

impl Condition for EqualsSubject {
    fn evaluate(&self, input: Box<RawValue>, req: &Request) -> bool {
        if let Ok(v) = serde_json::from_str::<String>(input.get()) {
            return req.subject == v;
        }
        false
    }
}
