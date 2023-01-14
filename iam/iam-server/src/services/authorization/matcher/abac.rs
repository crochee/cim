use cim_core::Result;

use crate::services::authorization::Matcher;

pub struct Abac;

impl Matcher for Abac {
    fn matches(
        &self,
        _delimiter_start: char,
        _delimiter_end: char,
        haystack: Vec<String>,
        needle: &str,
    ) -> Result<bool> {
        for v in haystack {
            if v == "*" || v.eq(needle) {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
