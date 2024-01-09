use tracing::debug;

use pim::Request;
use slo::{errors, Result};

/// authorize return  ok or error
pub async fn authorize<P, R>(
    policy: &P,
    matcher: &pim::Pim<R>,
    input: &Request,
) -> Result<()>
where
    P: storage::policies::PolicyStore,
    R: pim::Matcher,
{
    let list = policy.get_policy_by_request(input).await?;
    debug!("{:#?}", list);
    let mut statements = Vec::with_capacity(list.len());
    for policy in list.iter() {
        for statement in policy.statement.iter() {
            statements.push(statement.clone());
        }
    }
    matcher
        .is_allow(statements, input)
        .map_err(|err| errors::forbidden(&err.to_string()))
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use serde_json::json;

    use pim::*;
    use storage::policies;

    use super::authorize;

    #[tokio::test]
    async fn test_authorize() {
        let mut p = storage::policies::MockPolicyStore::new();
        p.expect_get_policy_by_request().returning(|_| {
            Ok(vec![policies::Policy {
                version: "1.0".to_owned(),
                statement: vec![Statement {
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
                        options: serde_json::value::to_raw_value(
                            &json!({"cidr": ["192.168.1.0/24"]}),
                        )
                        .unwrap(),
                    },
                ),
                (
                    "year".to_owned(),
                    JsonCondition {
                        jtype: "StringCmp".to_owned(),
                        options: serde_json::value::to_raw_value(&json!( {
                            "values": [{
                                "equal": true,
                                "ignore_case": false,
                                "value": "2023",
                            }],
                        }))
                        .unwrap(),
                    },
                ),
                (
                    "password".to_owned(),
                    JsonCondition {
                        jtype: "StringMatch".to_owned(),
                        options: serde_json::value::to_raw_value(&json!( {
                            "matches": "^[a-zA-Z][a-zA-Z0-9_#@\\$]{14,254}$",
                        }))
                        .unwrap(),
                    },
                ),
                (
                    "enable".to_owned(),
                    JsonCondition {
                        jtype: "Boolean".to_owned(),
                        options: serde_json::value::to_raw_value(&json!( {
                            "value": true,
                        }))
                        .unwrap(),
                    },
                ),
                (
                    "count".to_owned(),
                    JsonCondition {
                        jtype: "NumericCmp".to_owned(),
                        options: serde_json::value::to_raw_value(&json!( {
                            "symbol": ">",
                            "value": 5.0,
                        }))
                        .unwrap(),
                    },
                ),
                (
                    "login".to_owned(),
                    JsonCondition {
                        jtype: "TimeCmp".to_owned(),
                        options: serde_json::value::to_raw_value(&json!( {
                            "values": [{
                                "symbol": ">=",
                                "value": "10/01/2023 12:50",
                                "format": "%d/%m/%Y %H:%M",
                                "location":"LOCAL",
                            }],
                        }))
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
        }],
                id: "".to_owned(),
                account_id: None,
                desc: "test".to_owned(),
                created_at: chrono::NaiveDate::from_ymd_opt(2016, 7, 8)
                    .unwrap()
                    .and_hms_opt(9, 10, 11)
                    .unwrap(),
                updated_at: chrono::NaiveDate::from_ymd_opt(2016, 7, 8)
                    .unwrap()
                    .and_hms_opt(9, 10, 11)
                    .unwrap(),
            }])
        });

        let matcher = Pim::new(Regexp::new(256).unwrap());
        authorize(
            &p,
            &matcher,
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
        .await
        .unwrap();
    }
}
