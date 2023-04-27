use axum::{
    extract::Path,
    routing::{get, post},
    Json, Router,
};

use cim_core::Result;
use http::StatusCode;
use tracing::info;

use crate::{
    models::{
        self,
        usergroup::{UserGroup, UserGroupBindings},
    },
    pkg::valid::{Header, Valid},
    services::groups,
    store::{
        groups::{Content, Querys},
        Store,
    },
    var::SOURCE_SYSTEM,
    AppState,
};

pub struct GroupsRouter;

impl GroupsRouter {
    pub fn new_router(state: AppState) -> Router {
        Router::new()
            .route("/groups", get(Self::list).post(Self::create))
            .nest(
                "/groups/:id",
                Router::new()
                    .route(
                        "/",
                        get(Self::get).delete(Self::delete).put(Self::put),
                    )
                    .route(
                        "/users/:user_id",
                        post(Self::add_user).delete(Self::delete_user),
                    )
                    .route(
                        "/roles/:role_id",
                        post(Self::add_role).delete(Self::delete_role),
                    ),
            )
            .with_state(state)
    }

    async fn create(
        header: Header,
        app: AppState,
        Valid(Json(mut input)): Valid<Json<Content>>,
    ) -> Result<(StatusCode, Json<models::ID>)> {
        input.account_id = header.account_id;
        input.user_id = header.user_id;
        info!("list query {:#?}", input);
        let id = app.store.create_user_group(None, &input).await?;
        Ok((StatusCode::CREATED, id.into()))
    }

    async fn list(
        header: Header,
        app: AppState,
        Valid(mut filter): Valid<Querys>,
    ) -> Result<Json<models::List<UserGroup>>> {
        if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
            filter.account_id = Some(header.account_id);
        }
        filter.pagination.check();
        let list = app.store.list_user_group(&filter).await?;
        Ok(list.into())
    }

    async fn get(
        header: Header,
        app: AppState,
        Path(id): Path<String>,
    ) -> Result<Json<UserGroupBindings>> {
        let mut filter = Querys {
            account_id: None,
            pagination: Default::default(),
        };
        filter.pagination.check();
        filter.pagination.limit = 0;
        filter.pagination.offset = 0;
        if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
            filter.account_id = Some(header.account_id);
        }
        let resp = app.store.get_user_group(&id, &filter).await?;
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
        app.store.delete_user_group(&id, account_id).await?;
        Ok(StatusCode::NO_CONTENT)
    }

    async fn put(
        header: Header,
        app: AppState,
        Path(id): Path<String>,
        Valid(Json(mut content)): Valid<Json<Content>>,
    ) -> Result<StatusCode> {
        info!("list query {:#?} {:#?}", content, header);
        content.account_id = header.account_id;
        content.user_id = header.user_id;
        groups::put(&app, &id, &content).await?;
        Ok(StatusCode::NO_CONTENT)
    }

    async fn add_user(
        header: Header,
        app: AppState,
        Path((id, user_id)): Path<(String, String)>,
    ) -> Result<StatusCode> {
        info!("list query {:#?} {}", id, user_id);
        app.store
            .add_user_to_user_group(&id, &header.account_id, &user_id)
            .await?;
        Ok(StatusCode::NO_CONTENT)
    }

    async fn delete_user(
        header: Header,
        app: AppState,
        Path((id, user_id)): Path<(String, String)>,
    ) -> Result<StatusCode> {
        info!("list query {:#?} {} {}", header.account_id, id, user_id);
        app.store.delete_user_from_user_group(&id, &user_id).await?;
        Ok(StatusCode::NO_CONTENT)
    }

    async fn add_role(
        header: Header,
        app: AppState,
        Path((id, role_id)): Path<(String, String)>,
    ) -> Result<StatusCode> {
        info!("list query {:#?} {}", id, role_id);
        app.store
            .add_role_to_user_group(&id, &header.account_id, &role_id)
            .await?;
        Ok(StatusCode::NO_CONTENT)
    }

    async fn delete_role(
        header: Header,
        app: AppState,
        Path((id, role_id)): Path<(String, String)>,
    ) -> Result<StatusCode> {
        info!("list query {:#?} {} {}", header.account_id, id, role_id);
        app.store.delete_role_from_user_group(&id, &role_id).await?;
        Ok(StatusCode::NO_CONTENT)
    }
}
