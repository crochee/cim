use axum::{
    extract::Path,
    routing::{get, post},
    Json, Router,
};

use http::StatusCode;
use tracing::info;

use cim_slo::Result;
use cim_storage::{groups::*, List, ID};

use crate::{
    services::groups,
    valid::{Header, Valid},
    var::SOURCE_SYSTEM,
    AppState,
};

pub fn new_router(state: AppState) -> Router {
    Router::new()
        .route("/groups", get(list_group).post(create_group))
        .nest(
            "/groups/:id",
            Router::new()
                .route("/", get(get_group).delete(delete_group).put(put_group))
                .route("/users/:user_id", post(add_user).delete(delete_user)),
        )
        .with_state(state)
}

async fn create_group(
    header: Header,
    app: AppState,
    Valid(Json(mut input)): Valid<Json<Content>>,
) -> Result<(StatusCode, Json<ID>)> {
    input.account_id = header.account_id;
    info!("list query {:#?}", input);
    let id = app.store.group.create_group(None, &input).await?;
    Ok((StatusCode::CREATED, id.into()))
}

async fn list_group(
    header: Header,
    app: AppState,
    Valid(mut filter): Valid<ListOpts>,
) -> Result<Json<List<Group>>> {
    if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
        filter.account_id = Some(header.account_id);
    }
    let list = app.store.group.list_group(&filter).await?;
    Ok(list.into())
}

async fn get_group(
    header: Header,
    app: AppState,
    Path(id): Path<String>,
) -> Result<Json<Group>> {
    let mut account_id = None;
    if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
        account_id = Some(header.account_id);
    }
    let resp = app.store.group.get_group(&id, account_id).await?;
    Ok(resp.into())
}

async fn delete_group(
    header: Header,
    app: AppState,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    let mut account_id = None;
    if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
        account_id = Some(header.account_id);
    }
    app.store.group.delete_group(&id, account_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn put_group(
    header: Header,
    app: AppState,
    Path(id): Path<String>,
    Valid(Json(mut content)): Valid<Json<Content>>,
) -> Result<StatusCode> {
    info!("list query {:#?} {:#?}", content, header);
    content.account_id = header.account_id;
    groups::put_group(&app.store.group, &id, &content).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn add_user(
    header: Header,
    app: AppState,
    Path((id, user_id)): Path<(String, String)>,
) -> Result<StatusCode> {
    info!("list query {:#?} {}", id, user_id);
    app.store
        .group
        .attach_user(&id, Some(header.account_id), &user_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_user(
    header: Header,
    app: AppState,
    Path((id, user_id)): Path<(String, String)>,
) -> Result<StatusCode> {
    info!("list query {:#?} {} {}", header.account_id, id, user_id);
    app.store.group.detach_user(&id, &user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
