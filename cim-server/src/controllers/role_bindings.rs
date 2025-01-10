use axum::{extract::Path, response::Response, routing::get, Json, Router};
use http::StatusCode;

use cim_slo::{errors, next_id, Result};
use cim_storage::{
    role_binding::{Content, ListParams, RoleBinding},
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
            "/role_bindings",
            get(list_role_binding).post(create_role_binding),
        )
        .route(
            "/role_bindings/:id",
            get(get_role_binding)
                .delete(delete_role_binding)
                .put(put_role_binding),
        )
        .with_state(state)
}

async fn create_role_binding(
    _auth: Auth,
    app: AppState,
    Valid(Json(input)): Valid<Json<Content>>,
) -> Result<(StatusCode, Json<ID>)> {
    let id = next_id().map_err(errors::any)?;
    app.store
        .role_binding
        .create(&RoleBinding {
            id: id.to_string(),
            role_id: input.role_id,
            user_type: input.user_type,
            user_id: input.user_id,
            ..Default::default()
        })
        .await?;
    Ok((StatusCode::CREATED, ID { id: id.to_string() }.into()))
}

async fn list_role_binding(
    _auth: Auth,
    app: AppState,
    list_params: ListWatch<ListParams>,
) -> Result<Response> {
    list_watch(
        app.store.role_binding.clone(),
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

async fn get_role_binding(
    _auth: Auth,
    app: AppState,
    Path(id): Path<String>,
) -> Result<Json<RoleBinding>> {
    let mut result = RoleBinding {
        id: id.clone(),
        ..Default::default()
    };
    app.store.role_binding.get(&mut result).await?;
    Ok(result.into())
}

async fn delete_role_binding(
    _auth: Auth,
    app: AppState,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    let result = RoleBinding {
        id: id.clone(),
        ..Default::default()
    };
    app.store.role_binding.delete(&result).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn put_role_binding(
    _auth: Auth,
    app: AppState,
    Path(id): Path<String>,
    Valid(Json(content)): Valid<Json<Content>>,
) -> Result<StatusCode> {
    let mut result = RoleBinding {
        id: id.clone(),
        ..Default::default()
    };
    app.store.role_binding.get(&mut result).await?;

    result.role_id = content.role_id;
    result.user_type = content.user_type;
    result.user_id = content.user_id;
    app.store.role_binding.put(&result).await?;
    Ok(StatusCode::NO_CONTENT)
}
