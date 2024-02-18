use axum::{
    extract::Path,
    routing::{get, post},
    Json, Router,
};
use http::StatusCode;
use tracing::info;

use slo::Result;
use storage::{policies::*, List, ID};

use crate::{
    services::policies,
    valid::{Header, Valid},
    var::SOURCE_SYSTEM,
    AppState,
};

pub fn new_router(state: AppState) -> Router {
    Router::new()
        .route("/policies", get(list_policy).post(create_policy))
        .nest(
            "/policies/:id",
            Router::new()
                .route(
                    "/",
                    get(get_policy).delete(delete_policy).put(put_policy),
                )
                .route("/users/:user_id", post(attach_user).delete(detach_user))
                .route(
                    "/groups/:group_id",
                    post(attach_group).delete(detach_group),
                )
                .route(
                    "/roles/:role_id",
                    post(attach_role).delete(detach_role),
                ),
        )
        .with_state(state)
}

async fn create_policy(
    header: Header,
    app: AppState,
    Valid(Json(mut content)): Valid<Json<Content>>,
) -> Result<(StatusCode, Json<ID>)> {
    info!("list query {:#?} {:#?}", content, header);
    if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
        content.account_id = Some(header.account_id);
    }
    let id = app.store.policy.create_policy(None, &content).await?;
    Ok((StatusCode::CREATED, id.into()))
}

async fn list_policy(
    header: Header,
    app: AppState,
    Valid(mut filter): Valid<ListOpts>,
) -> Result<Json<List<Policy>>> {
    if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
        filter.account_id = Some(header.account_id);
    }
    let list = app.store.policy.list_policy(&filter).await?;
    Ok(list.into())
}

async fn get_policy(
    header: Header,
    app: AppState,
    Path(id): Path<String>,
) -> Result<Json<Policy>> {
    let mut account_id = None;
    if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
        account_id = Some(header.account_id);
    }
    let resp = app.store.policy.get_policy(&id, account_id).await?;
    Ok(resp.into())
}

async fn delete_policy(
    header: Header,
    app: AppState,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    let mut account_id = None;
    if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
        account_id = Some(header.account_id);
    }
    app.store.policy.delete_policy(&id, account_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn put_policy(
    header: Header,
    app: AppState,
    Path(id): Path<String>,
    Valid(Json(mut content)): Valid<Json<Content>>,
) -> Result<StatusCode> {
    info!("list query {:#?} {:#?}", content, header);
    if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
        content.account_id = Some(header.account_id);
    }
    policies::put_policy(&app.store.policy, &id, &content).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn attach_user(
    header: Header,
    app: AppState,
    Path((id, user_id)): Path<(String, String)>,
) -> Result<StatusCode> {
    let mut account_id = None;
    if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
        account_id = Some(header.account_id);
    }
    app.store
        .policy
        .attach(&id, account_id, &user_id, BindingsType::User)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn detach_user(
    _header: Header,
    app: AppState,
    Path((id, user_id)): Path<(String, String)>,
) -> Result<StatusCode> {
    app.store
        .policy
        .detach(&id, &user_id, BindingsType::User)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn attach_group(
    header: Header,
    app: AppState,
    Path((id, group_id)): Path<(String, String)>,
) -> Result<StatusCode> {
    let mut account_id = None;
    if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
        account_id = Some(header.account_id);
    }
    app.store
        .policy
        .attach(&id, account_id, &group_id, BindingsType::Group)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn detach_group(
    _header: Header,
    app: AppState,
    Path((id, group_id)): Path<(String, String)>,
) -> Result<StatusCode> {
    app.store
        .policy
        .detach(&id, &group_id, BindingsType::Group)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn attach_role(
    header: Header,
    app: AppState,
    Path((id, role_id)): Path<(String, String)>,
) -> Result<StatusCode> {
    let mut account_id = None;
    if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
        account_id = Some(header.account_id);
    }
    app.store
        .policy
        .attach(&id, account_id, &role_id, BindingsType::Role)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn detach_role(
    _header: Header,
    app: AppState,
    Path((id, role_id)): Path<(String, String)>,
) -> Result<StatusCode> {
    app.store
        .policy
        .detach(&id, &role_id, BindingsType::Role)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
