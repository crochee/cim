use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use validator::Validate;

use super::condition;

#[derive(Debug, Deserialize, Serialize, Validate, Clone)]
pub struct Statement {
    pub sid: Option<String>,
    pub effect: Effect,
    pub subject: Vec<String>,
    pub action: Vec<String>,
    pub resource: Vec<String>,
    pub condition: Option<HashMap<String, condition::JsonCondition>>,
    pub meta: Option<Vec<u8>>,
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

    use crate::models::condition::JsonCondition;

    use super::*;

    #[test]
    fn test_effect() {
        let r = serde_json::to_string(&Statement {
            sid: None,
            effect: Effect::Allow,
            subject: vec!["*".to_string()],
            action: vec!["*".to_string()],
            resource: vec!["*".to_string()],
            condition: Some(HashMap::from([(
                "Mercury".to_owned(),
                JsonCondition {
                    jtype: "vv".to_owned(),
                    options: vec![],
                },
            )])),
            meta: None,
        })
        .unwrap();
        println!("{}", r);
        let v: Policy = serde_json::from_value(json!(
            {
                "id": "ss",
                "desc": "FullAccess",
                "version": "v1.0.0",
                "statement": [
                    {
                        "effect": "Allow",
                        "subject": [
                            "user:*",
                            "account:*"
                        ],
                        "action": [
                            "*"
                        ],
                        "resource": [
                            "*"
                        ],
                        "collection": null,
                        "meta":null
                    }
                ]
            }
        ))
        .unwrap();
        println!("{:?}", v);
    }
}
