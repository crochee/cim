use axum::{extract::Path, routing::get, Json, Router};
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
        .route(
            "/policies/:id",
            get(get_policy).delete(delete_policy).put(put_policy),
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
