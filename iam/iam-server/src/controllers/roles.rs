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
        role::{Role, RoleBindings},
    },
    pkg::valid::{Header, Valid},
    services::roles,
    store::{
        roles::{Content, Querys},
        Store,
    },
    var::SOURCE_SYSTEM,
    AppState,
};

pub struct RolesRouter;

impl RolesRouter {
    pub fn new_router(state: AppState) -> Router {
        Router::new()
            .route("/roles", get(Self::list).post(Self::create))
            .nest(
                "/roles/:id",
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
                        "/policies/:policy_id",
                        post(Self::add_policy).delete(Self::delete_policy),
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
        let id = app.store.create_role(None, &input).await?;
        Ok((StatusCode::CREATED, id.into()))
    }

    async fn list(
        header: Header,
        app: AppState,
        Valid(mut filter): Valid<Querys>,
    ) -> Result<Json<models::List<Role>>> {
        if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
            filter.account_id = Some(header.account_id);
        }
        filter.pagination.check();
        let list = app.store.list_role(&filter).await?;
        Ok(list.into())
    }

    async fn get(
        header: Header,
        app: AppState,
        Path(id): Path<String>,
        Valid(mut filter): Valid<Querys>,
    ) -> Result<Json<RoleBindings>> {
        filter.pagination.check();
        if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
            filter.account_id = Some(header.account_id);
        }
        let resp = app.store.get_role(&id, &filter).await?;
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
        app.store.delete_role(&id, account_id).await?;
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
        roles::put(&app, &id, &content).await?;
        Ok(StatusCode::NO_CONTENT)
    }

    async fn add_user(
        header: Header,
        app: AppState,
        Path((id, user_id)): Path<(String, String)>,
    ) -> Result<StatusCode> {
        info!("list query {:#?} {}", id, user_id);
        app.store
            .add_user_to_role(&id, &header.account_id, &user_id)
            .await?;
        Ok(StatusCode::NO_CONTENT)
    }

    async fn delete_user(
        header: Header,
        app: AppState,
        Path((id, user_id)): Path<(String, String)>,
    ) -> Result<StatusCode> {
        info!("list query {:#?} {} {}", header.account_id, id, user_id);
        app.store.delete_user_from_role(&id, &user_id).await?;
        Ok(StatusCode::NO_CONTENT)
    }

    async fn add_policy(
        header: Header,
        app: AppState,
        Path((id, policy_id)): Path<(String, String)>,
    ) -> Result<StatusCode> {
        info!("list query {:#?} {}", id, policy_id);
        app.store
            .add_policy_to_role(&id, &header.account_id, &policy_id)
            .await?;
        Ok(StatusCode::NO_CONTENT)
    }

    async fn delete_policy(
        header: Header,
        app: AppState,
        Path((id, policy_id)): Path<(String, String)>,
    ) -> Result<StatusCode> {
        info!("list query {:#?} {} {}", header.account_id, id, policy_id);
        app.store.delete_policy_from_role(&id, &policy_id).await?;
        Ok(StatusCode::NO_CONTENT)
    }
}
