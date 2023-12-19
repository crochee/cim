use axum::{extract::Path, routing::get, Json, Router};

use http::StatusCode;
use tracing::info;

use crate::{
    models::{self, user::User},
    pkg::valid::{Header, Valid},
    services::users,
    store::{
        users::{Content, Querys},
        Store,
    },
    var::SOURCE_SYSTEM,
    AppState, Result,
};

pub struct UsersRouter;

impl UsersRouter {
    pub fn new_router(state: AppState) -> Router {
        Router::new()
            .route("/users", get(Self::list).post(Self::create))
            .route(
                "/users/:id",
                get(Self::get).delete(Self::delete).put(Self::put),
            )
            .with_state(state)
    }

    async fn create(
        app: AppState,
        Valid(Json(input)): Valid<Json<Content>>,
    ) -> Result<(StatusCode, Json<models::ID>)> {
        info!("list query {:#?}", input);
        let id = app.store.create_user(None, &input).await?;
        Ok((StatusCode::CREATED, id.into()))
    }

    async fn list(
        header: Header,
        app: AppState,
        Valid(mut filter): Valid<Querys>,
    ) -> Result<Json<models::List<User>>> {
        if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
            filter.account_id = Some(header.account_id);
        }
        filter.pagination.check();
        let list = app.store.list_user(&filter).await?;
        Ok(list.into())
    }

    async fn get(
        header: Header,
        app: AppState,
        Path(id): Path<String>,
    ) -> Result<Json<User>> {
        let mut account_id = None;
        if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
            account_id = Some(header.account_id);
        }
        let resp = app.store.get_user(&id, account_id).await?;
        Ok(resp.into())
    }

    async fn delete(
        header: Header,
        app: AppState,
        Path(id): Path<String>,
    ) -> Result<StatusCode> {
        let mut account_id = None;
        if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
            account_id = Some(header.account_id);
        }
        app.store.delete_user(&id, account_id).await?;
        Ok(StatusCode::NO_CONTENT)
    }

    async fn put(
        header: Header,
        app: AppState,
        Path(id): Path<String>,
        Valid(Json(mut content)): Valid<Json<Content>>,
    ) -> Result<StatusCode> {
        info!("list query {:#?} {:#?}", content, header);
        if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
            content.account_id = Some(header.account_id);
        }
        users::put(&app, &id, &content).await?;
        Ok(StatusCode::NO_CONTENT)
    }
}
