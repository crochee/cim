use std::collections::HashMap;

use axum::{extract::Path, response::Response, routing::get, Json, Router};
use http::StatusCode;

use cim_slo::{errors, next_id, Result};
use cim_storage::{
    policy::{Content, ListParams, Policy},
    Interface, WatchInterface, ID,
};

use crate::{
    auth::{Auth, Info},
    valid::{ListWatch, Valid},
    AppState,
};

use super::list_watch;

pub fn new_router(state: AppState) -> Router {
    Router::new()
        .route("/policies", get(list_policy).post(create_policy))
        .route(
            "/policies/{id}",
            get(get_policy).delete(delete_policy).put(put_policy),
        )
        .with_state(state)
}

async fn create_policy(
    auth: Auth,
    app: AppState,
    Valid(Json(content)): Valid<Json<Content>>,
) -> Result<(StatusCode, Json<ID>)> {
    let id = next_id().map_err(errors::any)?;
    app.store
        .policy
        .create(&Policy {
            id: id.to_string(),
            account_id: Some(auth.user.account_id),
            desc: content.desc,
            version: content.version,
            statement: content.statement,
            ..Default::default()
        })
        .await?;
    Ok((StatusCode::CREATED, ID { id: id.to_string() }.into()))
}

async fn list_policy(
    _auth: Auth,
    app: AppState,
    list_params: ListWatch<ListParams>,
) -> Result<Response> {
    list_watch(app.store.policy.clone(), list_params, |value, opts| {
        if let Some(ref v) = opts.id {
            if value.id.ne(v) {
                return true;
            }
        }
        if value.account_id != opts.account_id {
            return false;
        }
        false
    })
    .await
}

async fn get_policy(
    mut info: Info,
    app: AppState,
    Path(id): Path<String>,
) -> Result<Json<Policy>> {
    let mut result = Policy {
        id: id.clone(),
        ..Default::default()
    };
    app.store.policy.get(&mut result).await?;
    let mut opts = HashMap::new();
    if let Some(account_id) = &result.account_id {
        opts.insert("account_id".to_owned(), account_id.clone());
    }
    info.is_allow(&app.matcher, opts)?;
    Ok(result.into())
}

async fn delete_policy(
    mut info: Info,
    app: AppState,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    let mut result = Policy {
        id: id.clone(),
        ..Default::default()
    };
    app.store.policy.get(&mut result).await?;
    let mut opts = HashMap::new();
    if let Some(account_id) = &result.account_id {
        opts.insert("account_id".to_owned(), account_id.clone());
    }

    info.is_allow(&app.matcher, opts)?;
    app.store.policy.delete(&result).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn put_policy(
    mut info: Info,
    app: AppState,
    Path(id): Path<String>,
    Valid(Json(content)): Valid<Json<Content>>,
) -> Result<StatusCode> {
    let mut result = Policy {
        id: id.clone(),
        ..Default::default()
    };
    app.store.policy.get(&mut result).await?;
    let mut opts = HashMap::new();
    if let Some(account_id) = &result.account_id {
        opts.insert("account_id".to_owned(), account_id.clone());
    }
    info.is_allow(&app.matcher, opts)?;

    result.desc = content.desc;
    result.version = content.version;
    result.statement = content.statement;
    app.store.policy.put(&result).await?;
    Ok(StatusCode::NO_CONTENT)
}
