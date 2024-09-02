use std::collections::HashMap;

use rand::Rng;
use serde_json::json;

use cim_pim::{Effect, JsonCondition, Statement};
use cim_slo::{errors, next_id, Result};
use cim_storage::{
    client, connector, group, group_user, policy, policy_binding, user,
    Interface, WatchInterface,
};

use crate::AppState;

pub async fn create(app: AppState, input: user::Content) -> Result<u64> {
    if let Some(account_id) = &input.account_id {
        let mut user = user::User {
            id: account_id.to_owned(),
            ..Default::default()
        };
        app.store.user.get(&mut user).await?;
        let id = next_id().map_err(errors::any)?;
        app.store
            .user
            .create(&user::User {
                id: id.to_string(),
                account_id: account_id.to_owned(),
                desc: input.desc,
                claim: input.claim,
                secret: None,
                password: Some(input.password),
                ..Default::default()
            })
            .await?;
        return Ok(id);
    }
    let user_id = next_id().map_err(errors::any)?;
    app.store
        .user
        .create(&user::User {
            id: user_id.to_string(),
            account_id: user_id.to_string(),
            desc: input.desc,
            claim: input.claim,
            secret: None,
            password: Some(input.password),
            ..Default::default()
        })
        .await?;

    let group_id = next_id().map_err(errors::any)?;
    app.store
        .group
        .create(&group::Group {
            id: group_id.to_string(),
            account_id: user_id.to_string(),
            name: "Admin".to_owned(),
            desc: "Admin desc".to_owned(),
            ..Default::default()
        })
        .await?;

    let group_user_id = next_id().map_err(errors::any)?;
    app.store
        .group_user
        .create(&group_user::GroupUser {
            id: group_user_id.to_string(),
            group_id: group_id.to_string(),
            user_id: user_id.to_string(),
            ..Default::default()
        })
        .await?;
    let policy_id = next_id().map_err(errors::any)?;
    app.store
        .policy
        .create(&policy::Policy {
            id: policy_id.to_string(),
            account_id: Some(user_id.to_string()),
            desc: "Admin".to_owned(),
            version: "v1.0.0".to_owned(),
            statement: vec![Statement {
                effect: Effect::Allow,
                subjects: vec!["<.*>".to_owned()],
                actions: vec!["<.*>".to_owned()],
                resources: vec!["<.*>".to_owned()],
                conditions: Some(HashMap::from([(
                    "account_id".to_owned(),
                    JsonCondition {
                        jtype: "StringCmp".to_owned(),
                        options: serde_json::value::to_raw_value(&json!({
                            "values":[{
                                "equal": true,
                                "ignore_case": false,
                                "value": user_id.to_string(),
                            }]
                        }))
                        .unwrap(),
                    },
                )])),
                meta: None,
            }],
            ..Default::default()
        })
        .await?;

    let policy_binding_id = next_id().map_err(errors::any)?;
    app.store
        .policy_binding
        .create(&policy_binding::PolicyBinding {
            id: policy_binding_id.to_string(),
            policy_id: policy_id.to_string(),
            bindings_type: policy_binding::BindingsType::Group,
            bindings_id: group_id.to_string(),
            ..Default::default()
        })
        .await?;

    let connector_id = next_id().map_err(errors::any)?;
    app.store
        .connector
        .put(
            &connector::Connector {
                id: connector_id.to_string(),
                connector_type: "password".to_owned(),
                name: "owner".to_owned(),
                response_version: "v1.0.0".to_owned(),
                config: "{}".to_owned(),
                connector_data: None,
            },
            0,
        )
        .await?;

    let client_id = next_id().map_err(errors::any)?;
    let secret = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(12)
        .map(char::from)
        .collect::<String>();
    app.store
        .client
        .put(
            &client::Client {
                id: client_id.to_string(),
                secret,
                name: "owner".to_owned(),
                logo_url: "".to_owned(),
                account_id: user_id.to_string(),
                ..Default::default()
            },
            0,
        )
        .await?;

    Ok(user_id)
}
