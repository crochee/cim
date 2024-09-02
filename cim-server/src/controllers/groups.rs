use std::collections::HashMap;

use axum::{extract::Path, response::Response, routing::get, Json, Router};
use http::StatusCode;

use cim_slo::{errors, next_id, Result};
use cim_storage::{
    group::{Content, Group, ListParams},
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
        .route("/groups", get(list_group).post(create_group))
        .route(
            "/groups/:id",
            get(get_group).delete(delete_group).put(put_group),
        )
        .with_state(state)
}

async fn create_group(
    auth: Auth,
    app: AppState,
    Valid(Json(input)): Valid<Json<Content>>,
) -> Result<(StatusCode, Json<ID>)> {
    let id = next_id().map_err(errors::any)?;
    app.store
        .group
        .create(&Group {
            id: id.to_string(),
            account_id: auth.user.account_id,
            name: input.name,
            desc: input.desc,
            ..Default::default()
        })
        .await?;
    Ok((StatusCode::CREATED, ID { id: id.to_string() }.into()))
}

async fn list_group(
    _auth: Auth,
    app: AppState,
    list_params: ListWatch<ListParams>,
) -> Result<Response> {
    list_watch(app.store.group.clone(), list_params, |value, opts| {
        if let Some(ref v) = opts.id {
            if value.id.ne(v) {
                return true;
            }
        }
        if let Some(ref v) = opts.account_id {
            if value.account_id.ne(v) {
                return true;
            }
        }
        false
    })
    .await
}

async fn get_group(
    mut info: Info,
    app: AppState,
    Path(id): Path<String>,
) -> Result<Json<Group>> {
    let mut result = Group {
        id: id.clone(),
        ..Default::default()
    };
    app.store.group.get(&mut result).await?;
    info.is_allow(
        &app.matcher,
        HashMap::from([("account_id".to_owned(), result.account_id.clone())]),
    )?;
    Ok(result.into())
}

async fn delete_group(
    mut info: Info,
    app: AppState,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    let mut result = Group {
        id: id.clone(),
        ..Default::default()
    };
    app.store.group.get(&mut result).await?;
    info.is_allow(
        &app.matcher,
        HashMap::from([("account_id".to_owned(), result.account_id.clone())]),
    )?;
    app.store.group.delete(&result).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn put_group(
    mut info: Info,
    app: AppState,
    Path(id): Path<String>,
    Valid(Json(content)): Valid<Json<Content>>,
) -> Result<StatusCode> {
    let mut result = Group {
        id: id.clone(),
        ..Default::default()
    };
    app.store.group.get(&mut result).await?;
    info.is_allow(
        &app.matcher,
        HashMap::from([("account_id".to_owned(), result.account_id.clone())]),
    )?;

    result.name = content.name;
    result.desc = content.desc;
    app.store.group.put(&result, 0).await?;
    Ok(StatusCode::NO_CONTENT)
}
