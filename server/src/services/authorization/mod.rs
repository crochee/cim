pub mod matcher;

use crate::{errors, Result};

use crate::{
    models::{
        policy::{Effect, Policy, Statement},
        req::Request,
    },
    store::Store,
    AppState,
};

pub trait Matcher {
    fn matches(
        &self,
        delimiter_start: char,
        delimiter_end: char,
        haystack: Vec<String>,
        needle: &str,
    ) -> Result<bool>;
}

/// authorize return  ok or error
pub async fn authorize(app: &AppState, input: &Request) -> Result<()> {
    let list = app.store.get_policy_by_user(&input.subject).await?;
    tracing::debug!("{:#?}", list);
    match_policies(&app.matcher, list, input)
}

fn match_policies<T: Matcher>(
    matcher: &T,
    list: Vec<Policy>,
    input: &Request,
) -> Result<()> {
    let mut allowed = false;
    for policy in list.iter() {
        for statement in policy.statement.iter() {
            let am = matcher.matches(
                statement.get_start_delimiter(),
                statement.get_end_delimiter(),
                statement.actions.clone(),
                &input.action,
            )?;
            if !am {
                continue;
            }
            let sm = matcher.matches(
                statement.get_start_delimiter(),
                statement.get_end_delimiter(),
                statement.subjects.clone(),
                &input.subject,
            )?;
            if !sm {
                continue;
            }
            let rm = matcher.matches(
                statement.get_start_delimiter(),
                statement.get_end_delimiter(),
                statement.resources.clone(),
                &input.resource,
            )?;
            if !rm {
                continue;
            }
            let cm = evaluate_conditions(statement, input)?;
            if !cm {
                continue;
            }
            if let Effect::Deny = statement.effect {
                return Err(errors::forbidden(&format!("The request was denied because a policy denied request.Please proofread the policy {}",policy.id)));
            }
            allowed = true;
        }
    }
    if !allowed {
        return Err(errors::forbidden(
            "The request was denied because no matching policy was found.",
        ));
    }
    Ok(())
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
    use std::{collections::HashMap, num::NonZeroUsize, sync::Mutex};

    use chrono::{Local, NaiveDate};

    use crate::{
        models::condition::{
            Boolean, Cidr, EqualsSubject, JsonCondition, NumericCmp, StringCmp,
            StringCmpInner, StringMatch, TimeCmp, TimeCmpInner,
        },
        services::authorization::matcher::reg::Regexp,
    };

    use super::*;

    #[test]
    fn auth_match() {
        let pols=vec![
            Policy {
            id: "1".to_owned(),
            account_id: None,
            user_id: None,
            desc: "This policy allows max, peter, zac and ken to create, delete and get the listed resources,
            but only if the client ip matches and the request states that they are the owner of those resources as well.".to_owned(),
            version: "20230114".to_owned(),
            statement: vec![
                Statement {
                    sid: None,
                    effect: Effect::Allow,
                    subjects: vec!["max".to_owned(), "peter".to_owned(), "<zac|ken>".to_owned()],
                    actions: vec!["<create|delete>".to_owned(), "get".to_owned()],
                    resources: vec!["myrn:some.domain.com:resource:123".to_owned(), "myrn:some.domain.com:resource:345".to_owned(), "myrn:something:foo:<.+>".to_owned(),"myrn:some.domain.com:resource:<\\d+>".to_owned()],
                    conditions: Some(HashMap::from([
                        ("owner".to_owned(), JsonCondition{ 
                            jtype: "EqualsSubject".to_owned(),
                            options: serde_json::value::to_raw_value(&EqualsSubject{}).unwrap() ,
                        }),
                        ("clientIP".to_owned(), JsonCondition{ 
                            jtype: "CIDR".to_owned(),
                            options: serde_json::value::to_raw_value(&Cidr{ cidr: vec!["192.168.1.0/24".to_owned()] }).unwrap(),
                         }),
                         ("year".to_owned(), JsonCondition{ 
                            jtype: "StringCmp".to_owned(),
                            options: serde_json::value::to_raw_value(&StringCmp{ values: vec![StringCmpInner{
                                equal:true,ignore_case:false,value:"2023".to_owned(),
                            }] }).unwrap(),
                         }),
                         ("password".to_owned(), JsonCondition{ 
                            jtype: "StringMatch".to_owned(),
                            options: serde_json::value::to_raw_value(&StringMatch{matches:r"^[a-zA-Z][a-zA-Z0-9_#@\\$]{14,254}$".to_owned() }).unwrap(),
                         }),
                         ("enable".to_owned(), JsonCondition{ 
                            jtype: "Boolean".to_owned(),
                            options: serde_json::value::to_raw_value(&Boolean{value:true }).unwrap(),
                         }),
                         ("count".to_owned(), JsonCondition{ 
                            jtype: "NumericCmp".to_owned(),
                            options: serde_json::value::to_raw_value(&NumericCmp{ symbol:">".to_owned(),value:serde_json::Number::from_f64(5.0).unwrap()}).unwrap(),
                         }),
                         ("login".to_owned(), JsonCondition{ 
                            jtype: "TimeCmp".to_owned(),
                            options: serde_json::value::to_raw_value(&TimeCmp{ values:vec![TimeCmpInner{
                                symbol: ">=".to_owned(),
                                value: "15/01/2023 12:50".to_owned(),
                                format: "%d/%m/%Y %H:%M".to_owned(),
                                location: Some("LOCAL".to_owned()),
                                }]}).unwrap(),
                         }),
                        ])),
                    meta: None,
                },
            ],
            created_at:NaiveDate::from_ymd_opt(2016, 7, 8).unwrap().and_hms_opt(9, 10, 11).unwrap(),
            updated_at:NaiveDate::from_ymd_opt(2016, 7, 8).unwrap().and_hms_opt(9, 10, 11).unwrap(),
        },
        ];

        let a = Regexp {
            lru: Mutex::new(lru::LruCache::new(
                NonZeroUsize::new(256).unwrap(),
            )),
        };
        match_policies(
            &a,
            pols,
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
                        serde_json::value::to_raw_value(
                            &Local::now().format("%d/%m/%Y %H:%M").to_string(),
                        )
                        .unwrap(),
                    ),
                ]),
            },
        )
        .unwrap();
    }
}
