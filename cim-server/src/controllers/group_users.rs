use axum::{extract::Path, response::Response, routing::get, Json, Router};
use http::StatusCode;

use cim_slo::{errors, next_id, Result};
use cim_storage::{
    group_user::{Content, GroupUser, ListParams},
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
        .route("/group_users", get(list_group_user).post(create_group_user))
        .route(
            "/group_users/:id",
            get(get_group_user)
                .delete(delete_group_user)
                .put(put_group_user),
        )
        .with_state(state)
}

async fn create_group_user(
    _auth: Auth,
    app: AppState,
    Valid(Json(input)): Valid<Json<Content>>,
) -> Result<(StatusCode, Json<ID>)> {
    let id = next_id().map_err(errors::any)?;
    app.store
        .group_user
        .create(&GroupUser {
            id: id.to_string(),
            group_id: input.group_id,
            user_id: input.user_id,
            ..Default::default()
        })
        .await?;
    Ok((StatusCode::CREATED, ID { id: id.to_string() }.into()))
}

async fn list_group_user(
    _auth: Auth,
    app: AppState,
    list_params: ListWatch<ListParams>,
) -> Result<Response> {
    list_watch(app.store.group_user.clone(), list_params, |value, opts| {
        if let Some(ref v) = opts.id {
            if value.id.ne(v) {
                return true;
            }
        }
        false
    })
    .await
}

async fn get_group_user(
    _auth: Auth,
    app: AppState,
    Path(id): Path<String>,
) -> Result<Json<GroupUser>> {
    let mut result = GroupUser {
        id: id.clone(),
        ..Default::default()
    };
    app.store.group_user.get(&mut result).await?;
    Ok(result.into())
}

async fn delete_group_user(
    _auth: Auth,
    app: AppState,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    let result = GroupUser {
        id: id.clone(),
        ..Default::default()
    };
    app.store.group_user.delete(&result).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn put_group_user(
    _auth: Auth,
    app: AppState,
    Path(id): Path<String>,
    Valid(Json(content)): Valid<Json<Content>>,
) -> Result<StatusCode> {
    let mut result = GroupUser {
        id: id.clone(),
        ..Default::default()
    };
    app.store.group_user.get(&mut result).await?;

    result.user_id = content.user_id;
    result.group_id = content.group_id;
    app.store.group_user.put(&result, 0).await?;
    Ok(StatusCode::NO_CONTENT)
}
