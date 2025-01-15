use std::collections::HashMap;

use axum::{extract::Path, response::Response, routing::get, Json, Router};
use http::StatusCode;

use cim_slo::Result;
use cim_storage::{
    user::{Content, ListParams, User},
    Interface, ID,
};

use crate::{
    auth::{Auth, Info},
    services::user,
    valid::{ListWatch, Valid},
    AppState,
};

use super::list_watch;

pub fn new_router(state: AppState) -> Router {
    Router::new()
        .route("/users", get(list_user).post(create_user))
        .route(
            "/users/{id}",
            get(get_user).delete(delete_user).put(put_user),
        )
        .with_state(state)
}

async fn create_user(
    app: AppState,
    Valid(Json(input)): Valid<Json<Content>>,
) -> Result<(StatusCode, Json<ID>)> {
    let id = user::create(app, input).await?;
    Ok((StatusCode::CREATED, ID { id: id.to_string() }.into()))
}

async fn list_user(
    _auth: Auth,
    app: AppState,
    list_params: ListWatch<ListParams>,
) -> Result<Response> {
    list_watch(app.store.user.clone(), list_params, |value, opts| {
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

async fn get_user(
    mut info: Info,
    app: AppState,
    Path(id): Path<String>,
) -> Result<Json<User>> {
    let mut result = User {
        id: id.clone(),
        ..Default::default()
    };
    app.store.user.get(&mut result).await?;
    info.is_allow(
        &app.matcher,
        HashMap::from([("account_id".to_owned(), result.account_id.clone())]),
    )?;
    result.secret = None;
    result.password = None;
    Ok(result.into())
}

async fn delete_user(
    mut info: Info,
    app: AppState,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    let mut result = User {
        id: id.clone(),
        ..Default::default()
    };
    app.store.user.get(&mut result).await?;
    info.is_allow(
        &app.matcher,
        HashMap::from([("account_id".to_owned(), result.account_id.clone())]),
    )?;
    app.store.user.delete(&result).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn put_user(
    mut info: Info,
    app: AppState,
    Path(id): Path<String>,
    Valid(Json(content)): Valid<Json<Content>>,
) -> Result<StatusCode> {
    let mut user = User {
        id: id.clone(),
        ..Default::default()
    };
    app.store.user.get(&mut user).await?;
    info.is_allow(
        &app.matcher,
        HashMap::from([("account_id".to_owned(), user.account_id.clone())]),
    )?;
    user.desc = content.desc;
    user.claim = content.claim;
    user.password = Some(content.password);
    app.store.user.put(&user).await?;
    Ok(StatusCode::NO_CONTENT)
}
