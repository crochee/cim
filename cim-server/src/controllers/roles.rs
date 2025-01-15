use std::collections::HashMap;

use axum::{extract::Path, response::Response, routing::get, Json, Router};
use http::StatusCode;
use tracing::info;

use cim_slo::{errors, next_id, Result};
use cim_storage::{
    role::{Content, ListParams, Role},
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
        .route("/roles", get(list_role).post(create_role))
        .route(
            "/roles/{id}",
            get(get_role).delete(delete_role).put(put_role),
        )
        .with_state(state)
}

async fn create_role(
    auth: Auth,
    app: AppState,
    Valid(Json(input)): Valid<Json<Content>>,
) -> Result<(StatusCode, Json<ID>)> {
    info!("list query {:#?}", input);
    let id = next_id().map_err(errors::any)?;
    app.store
        .role
        .create(&Role {
            id: id.to_string(),
            account_id: auth.user.account_id,
            name: input.name,
            desc: input.desc,
            ..Default::default()
        })
        .await?;
    Ok((StatusCode::CREATED, ID { id: id.to_string() }.into()))
}

async fn list_role(
    _auth: Auth,
    app: AppState,
    list_params: ListWatch<ListParams>,
) -> Result<Response> {
    list_watch(app.store.role.clone(), list_params, |value, opts| {
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

async fn get_role(
    mut info: Info,
    app: AppState,
    Path(id): Path<String>,
) -> Result<Json<Role>> {
    let mut result = Role {
        id: id.clone(),
        ..Default::default()
    };
    app.store.role.get(&mut result).await?;
    info.is_allow(
        &app.matcher,
        HashMap::from([("account_id".to_owned(), result.account_id.clone())]),
    )?;
    Ok(result.into())
}

async fn delete_role(
    mut info: Info,
    app: AppState,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    let mut result = Role {
        id: id.clone(),
        ..Default::default()
    };
    app.store.role.get(&mut result).await?;
    info.is_allow(
        &app.matcher,
        HashMap::from([("account_id".to_owned(), result.account_id.clone())]),
    )?;
    app.store.role.delete(&result).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn put_role(
    mut info: Info,
    app: AppState,
    Path(id): Path<String>,
    Valid(Json(content)): Valid<Json<Content>>,
) -> Result<StatusCode> {
    let mut role = Role {
        id: id.clone(),
        ..Default::default()
    };
    app.store.role.get(&mut role).await?;
    info.is_allow(
        &app.matcher,
        HashMap::from([("account_id".to_owned(), role.account_id.clone())]),
    )?;
    role.name = content.name;
    role.desc = content.desc;
    app.store.role.put(&role).await?;
    Ok(StatusCode::NO_CONTENT)
}
