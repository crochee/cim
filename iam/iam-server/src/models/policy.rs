use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use validator::Validate;

use super::condition;

#[derive(Debug, Deserialize, Serialize, Validate, Clone)]
pub struct Statement {
    pub sid: Option<String>,
    pub effect: Effect,
    pub subjects: Vec<String>,
    pub actions: Vec<String>,
    pub resources: Vec<String>,
    pub conditions: Option<HashMap<String, condition::JsonCondition>>,
    pub meta: Option<Box<RawValue>>,
}

impl Statement {
    pub fn get_start_delimiter(&self) -> char {
        '<'
    }

    pub fn get_end_delimiter(&self) -> char {
        '>'
    }
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct Policy {
    pub id: String,
    pub account_id: Option<String>,
    pub user_id: Option<String>,
    #[validate(length(min = 1))]
    pub desc: String,
    // 指定要使用的策略语言版本
    pub version: String,
    pub statement: Vec<Statement>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Effect {
    Allow,
    Deny,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_effect() {
        let v: Policy = serde_json::from_value(json!(
            {
                "id": "123",
                "desc": "This policy allows max, peter, zac and ken to create, delete and get the listed resources,but only if the client ip matches and the request states that they are the owner of those resources as well.",
                "version": "20230114",
                "statement": [
                    {
                        "effect": "Allow",
                        "subject": [
                            "max",
                            "peter",
                            "<zac|ken>"
                        ],
                        "action": [
                            "<create|delete>",
                            "get"
                        ],
                        "resource": [
                            "myrn:some.domain.com:resource:<d+>"
                        ],
                        "condition": {
                            "owner": {
                                "type": "EqualsSubjectCondition",
                                "options": {}
                            },
                            "clientIP": {
                                "type": "CIDRCondition",
                                "options": {
                                    "cidr": "192.168.1.0/24"
                                }
                            },
                            "year": {
                                "type": "StringEqualCondition",
                                "options": {
                                    "equals": "2023"
                                }
                            },
                            "password": {
                                "type": "StringMatchCondition",
                                "options": {
                                    "matches": "^[a-zA-Z][a-zA-Z0-9_#@\\\\$]{14,254}$"
                                }
                            }
                        },
                        "meta": null
                    }
                ]
            }
        ))
        .unwrap();
        println!("{:?}", v);
    }
}
