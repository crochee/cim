use axum::{extract::Path, routing::get, Json, Router};

use http::StatusCode;
use tracing::info;

use slo::Result;
use storage::{users::*, List, ID};

use crate::{
    services::users,
    valid::{Header, Valid},
    var::SOURCE_SYSTEM,
    AppState,
};

pub fn new_router(state: AppState) -> Router {
    Router::new()
        .route("/users", get(list_user).post(create_user))
        .route(
            "/users/:id",
            get(get_user).delete(delete_user).put(put_user),
        )
        .with_state(state)
}

async fn create_user(
    app: AppState,
    Valid(Json(input)): Valid<Json<Content>>,
) -> Result<(StatusCode, Json<ID>)> {
    info!("list query {:#?}", input);
    let id = app.store.user.create_user(None, &input).await?;
    Ok((StatusCode::CREATED, id.into()))
}

async fn list_user(
    header: Header,
    app: AppState,
    Valid(mut filter): Valid<ListOpts>,
) -> Result<Json<List<User>>> {
    if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
        filter.account_id = Some(header.account_id);
    }
    let list = app.store.user.list_user(&filter).await?;
    Ok(list.into())
}

async fn get_user(
    header: Header,
    app: AppState,
    Path(id): Path<String>,
) -> Result<Json<User>> {
    let mut account_id = None;
    if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
        account_id = Some(header.account_id);
    }
    let resp = app.store.user.get_user(&id, account_id).await?;
    Ok(resp.into())
}

async fn delete_user(
    header: Header,
    app: AppState,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    let mut account_id = None;
    if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
        account_id = Some(header.account_id);
    }
    app.store.user.delete_user(&id, account_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn put_user(
    header: Header,
    app: AppState,
    Path(id): Path<String>,
    Valid(Json(mut content)): Valid<Json<Content>>,
) -> Result<StatusCode> {
    info!("list query {:#?} {:#?}", content, header);
    if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
        content.account_id = Some(header.account_id);
    }
    users::put_user(&app.store.user, &id, &content).await?;
    Ok(StatusCode::NO_CONTENT)
}
