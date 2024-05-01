use axum::{
    extract::Path,
    routing::{get, post},
    Json, Router,
};

use http::StatusCode;
use tracing::info;

use cim_slo::Result;
use cim_storage::{roles::*, List, ID};

use crate::{
    services::roles,
    valid::{Header, Valid},
    var::SOURCE_SYSTEM,
    AppState,
};

pub fn new_router(state: AppState) -> Router {
    Router::new()
        .route("/roles", get(list_role).post(create_role))
        .nest(
            "/roles/:id",
            Router::new()
                .route("/", get(get_role).delete(delete_role).put(put_role))
                .route("/users/:user_id", post(add_user).delete(delete_user)),
        )
        .with_state(state)
}

async fn create_role(
    header: Header,
    app: AppState,
    Valid(Json(mut input)): Valid<Json<Content>>,
) -> Result<(StatusCode, Json<ID>)> {
    input.account_id = header.account_id;
    info!("list query {:#?}", input);
    let id = app.store.role.create_role(None, &input).await?;
    Ok((StatusCode::CREATED, id.into()))
}

async fn list_role(
    header: Header,
    app: AppState,
    Valid(mut filter): Valid<ListOpts>,
) -> Result<Json<List<Role>>> {
    if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
        filter.account_id = Some(header.account_id);
    }
    let list = app.store.role.list_role(&filter).await?;
    Ok(list.into())
}

async fn get_role(
    header: Header,
    app: AppState,
    Path(id): Path<String>,
) -> Result<Json<Role>> {
    let mut account_id = None;
    if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
        account_id = Some(header.account_id);
    }
    let resp = app.store.role.get_role(&id, account_id).await?;
    Ok(resp.into())
}

async fn delete_role(
    header: Header,
    app: AppState,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    let mut account_id = None;
    if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
        account_id = Some(header.account_id);
    }
    app.store.role.delete_role(&id, account_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn put_role(
    header: Header,
    app: AppState,
    Path(id): Path<String>,
    Valid(Json(mut content)): Valid<Json<Content>>,
) -> Result<StatusCode> {
    info!("list query {:#?} {:#?}", content, header);
    content.account_id = header.account_id;
    roles::put_role(&app.store.role, &id, &content).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn add_user(
    header: Header,
    app: AppState,
    Path((id, user_id)): Path<(String, String)>,
) -> Result<StatusCode> {
    info!("list query {:#?} {}", id, user_id);
    app.store
        .role
        .attach_user(&id, Some(header.account_id), &user_id, UserType::User)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_user(
    header: Header,
    app: AppState,
    Path((id, user_id)): Path<(String, String)>,
) -> Result<StatusCode> {
    info!("list query {:#?} {} {}", header.account_id, id, user_id);
    app.store
        .role
        .detach_user(&id, &user_id, UserType::User)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
