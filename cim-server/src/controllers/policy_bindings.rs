use axum::{extract::Path, response::Response, routing::get, Json, Router};
use http::StatusCode;

use cim_slo::{errors, next_id, Result};
use cim_storage::{
    policy_binding::{Content, ListParams, PolicyBinding},
    Interface, WatchInterface, ID,
};

use crate::{
    auth::Auth,
    valid::{ListWatch, Valid},
    AppState,
};

use super::list_watch;

pub fn new_router(state: AppState) -> Router {
    Router::new()
        .route(
            "/policy_bindings",
            get(list_policy_binding).post(create_policy_binding),
        )
        .route(
            "/policy_bindings/:id",
            get(get_policy_binding)
                .delete(delete_policy_binding)
                .put(put_policy_binding),
        )
        .with_state(state)
}

async fn create_policy_binding(
    _auth: Auth,
    app: AppState,
    Valid(Json(input)): Valid<Json<Content>>,
) -> Result<(StatusCode, Json<ID>)> {
    let id = next_id().map_err(errors::any)?;
    app.store
        .policy_binding
        .create(&PolicyBinding {
            id: id.to_string(),
            policy_id: input.policy_id,
            bindings_type: input.bindings_type,
            bindings_id: input.bindings_id,
            ..Default::default()
        })
        .await?;
    Ok((StatusCode::CREATED, ID { id: id.to_string() }.into()))
}

async fn list_policy_binding(
    _auth: Auth,
    app: AppState,
    list_params: ListWatch<ListParams>,
) -> Result<Response> {
    list_watch(
        app.store.policy_binding.clone(),
        list_params,
        |value, opts| {
            if let Some(ref v) = opts.id {
                if value.id.ne(v) {
                    return true;
                }
            }
            false
        },
    )
    .await
}

async fn get_policy_binding(
    _auth: Auth,
    app: AppState,
    Path(id): Path<String>,
) -> Result<Json<PolicyBinding>> {
    let mut result = PolicyBinding {
        id: id.clone(),
        ..Default::default()
    };
    app.store.policy_binding.get(&mut result).await?;
    Ok(result.into())
}

async fn delete_policy_binding(
    _auth: Auth,
    app: AppState,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    let result = PolicyBinding {
        id: id.clone(),
        ..Default::default()
    };
    app.store.policy_binding.delete(&result).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn put_policy_binding(
    _auth: Auth,
    app: AppState,
    Path(id): Path<String>,
    Valid(Json(content)): Valid<Json<Content>>,
) -> Result<StatusCode> {
    let mut result = PolicyBinding {
        id: id.clone(),
        ..Default::default()
    };
    app.store.policy_binding.get(&mut result).await?;

    result.policy_id = content.policy_id;
    result.bindings_type = content.bindings_type;
    result.bindings_id = content.bindings_id;
    app.store.policy_binding.put(&result).await?;
    Ok(StatusCode::NO_CONTENT)
}
