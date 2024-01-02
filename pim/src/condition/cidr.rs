use std::net::IpAddr;
use std::str::FromStr;

use cidr_utils::cidr::IpCidr;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

use super::Condition;
use crate::req::Request;

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
                        if !cidr.contains(&ip) {
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
