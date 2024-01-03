mod condition;
pub mod matcher;
pub mod req;
pub mod statement;

use anyhow::Result;

use matcher::Matcher;
use req::Request;
use statement::{Effect, Statement};

pub struct Pim<M> {
    matcher: M,
}

impl<M> Pim<M> {
    pub fn new(matcher: M) -> Self {
        Self { matcher }
    }
}

impl<M: Matcher> Pim<M> {
    pub fn is_allow(
        &self,
        list: Vec<Statement>,
        input: &Request,
    ) -> Result<()> {
        let mut allowed = false;
        for statement in list.iter() {
            if !self.matcher.matches(
                statement.get_start_delimiter(),
                statement.get_end_delimiter(),
                statement.actions.clone(),
                &input.action,
            )? {
                continue;
            }
            if !self.matcher.matches(
                statement.get_start_delimiter(),
                statement.get_end_delimiter(),
                statement.subjects.clone(),
                &input.subject,
            )? {
                continue;
            }
            if !self.matcher.matches(
                statement.get_start_delimiter(),
                statement.get_end_delimiter(),
                statement.resources.clone(),
                &input.resource,
            )? {
                continue;
            }
            if !evaluate_conditions(statement, input)? {
                continue;
            }
            if let Effect::Deny = statement.effect {
                return Err(anyhow::anyhow!("The request was denied because a statement denied request.Please proofread the policy {:?}",statement));
            }
            allowed = true;
        }
        if !allowed {
            return Err(anyhow::anyhow!(
                "The request was denied because no matching statement was found.",
            ));
        }
        Ok(())
    }
}

fn evaluate_conditions(statement: &Statement, input: &Request) -> Result<bool> {
    if let Some(conditions) = &statement.conditions {
        for (key, value) in conditions {
            if let Some(env) = input.context.get(key) {
                let condition = value.into()?;
                if !condition.evaluate(env.clone(), input) {
                    return Ok(false);
                }
            }
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        condition::{
            boolean::Boolean, cidr::Cidr, numeric_cmp::NumericCmp,
            string_cmp::StringCmp, string_cmp::StringCmpInner,
            string_match::StringMatch, time_cmp::TimeCmp,
            time_cmp::TimeCmpInner, JsonCondition,
        },
        matcher::reg::Regexp,
    };

    use super::*;

    #[test]
    fn is_allow() {
        let sts = vec![Statement {
            sid: None,
            effect: Effect::Allow,
            subjects: vec![
                "max".to_owned(),
                "peter".to_owned(),
                "<zac|ken>".to_owned(),
            ],
            actions: vec!["<create|delete>".to_owned(), "get".to_owned()],
            resources: vec![
                "myrn:some.domain.com:resource:123".to_owned(),
                "myrn:some.domain.com:resource:345".to_owned(),
                "myrn:something:foo:<.+>".to_owned(),
                "myrn:some.domain.com:resource:<\\d+>".to_owned(),
            ],
            conditions: Some(HashMap::from([
                (
                    "owner".to_owned(),
                    JsonCondition {
                        jtype: "EqualsSubject".to_owned(),
                        options: serde_json::value::to_raw_value("{}").unwrap(),
                    },
                ),
                (
                    "clientIP".to_owned(),
                    JsonCondition {
                        jtype: "CIDR".to_owned(),
                        options: serde_json::value::to_raw_value(&Cidr {
                            cidr: vec!["192.168.1.0/24".to_owned()],
                        })
                        .unwrap(),
                    },
                ),
                (
                    "year".to_owned(),
                    JsonCondition {
                        jtype: "StringCmp".to_owned(),
                        options: serde_json::value::to_raw_value(&StringCmp {
                            values: vec![StringCmpInner {
                                equal: true,
                                ignore_case: false,
                                value: "2023".to_owned(),
                            }],
                        })
                        .unwrap(),
                    },
                ),
                (
                    "password".to_owned(),
                    JsonCondition {
                        jtype: "StringMatch".to_owned(),
                        options: serde_json::value::to_raw_value(
                            &StringMatch {
                                matches: r"^[a-zA-Z][a-zA-Z0-9_#@\\$]{14,254}$"
                                    .to_owned(),
                            },
                        )
                        .unwrap(),
                    },
                ),
                (
                    "enable".to_owned(),
                    JsonCondition {
                        jtype: "Boolean".to_owned(),
                        options: serde_json::value::to_raw_value(&Boolean {
                            value: true,
                        })
                        .unwrap(),
                    },
                ),
                (
                    "count".to_owned(),
                    JsonCondition {
                        jtype: "NumericCmp".to_owned(),
                        options: serde_json::value::to_raw_value(&NumericCmp {
                            symbol: ">".to_owned(),
                            value: serde_json::Number::from_f64(5.0).unwrap(),
                        })
                        .unwrap(),
                    },
                ),
                (
                    "login".to_owned(),
                    JsonCondition {
                        jtype: "TimeCmp".to_owned(),
                        options: serde_json::value::to_raw_value(&TimeCmp {
                            values: vec![TimeCmpInner {
                                symbol: ">=".to_owned(),
                                value: "10/01/2023 12:50".to_owned(),
                                format: "%d/%m/%Y %H:%M".to_owned(),
                                location: Some("LOCAL".to_owned()),
                            }],
                        })
                        .unwrap(),
                    },
                ),
                (
                    "resource".to_owned(),
                    JsonCondition {
                        jtype: "ResourceContains".to_owned(),
                        options: serde_json::value::to_raw_value("{}").unwrap(),
                    },
                ),
            ])),
            meta: None,
        }];

        let p = super::Pim::new(Regexp::new(256));
        p.is_allow(
            sts,
            &Request {
                resource: "myrn:some.domain.com:resource:123".to_owned(),
                action: "delete".to_owned(),
                subject: "peter".to_owned(),
                context: HashMap::from([
                    (
                        "owner".to_owned(),
                        serde_json::value::to_raw_value("peter").unwrap(),
                    ),
                    (
                        "clientIP".to_owned(),
                        serde_json::value::to_raw_value("192.168.1.67")
                            .unwrap(),
                    ),
                    (
                        "year".to_owned(),
                        serde_json::value::to_raw_value("2023").unwrap(),
                    ),
                    (
                        "password".to_owned(),
                        serde_json::value::to_raw_value("a12345678901234567")
                            .unwrap(),
                    ),
                    (
                        "enable".to_owned(),
                        serde_json::value::to_raw_value(&true).unwrap(),
                    ),
                    (
                        "count".to_owned(),
                        serde_json::value::to_raw_value(&6.0).unwrap(),
                    ),
                    (
                        "login".to_owned(),
                        serde_json::value::to_raw_value("15/01/2023 12:50")
                            .unwrap(),
                    ),
                    (
                        "resource".to_owned(),
                        serde_json::value::to_raw_value(&HashMap::from([
                            ("value".to_owned(), "123".to_owned()),
                            ("delimiter".to_owned(), "".to_owned()),
                        ]))
                        .unwrap(),
                    ),
                ]),
            },
        )
        .unwrap();
    }
}
