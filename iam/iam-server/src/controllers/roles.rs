use axum::{
    extract::Path,
    routing::{get, post},
    Extension, Json, Router,
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
    repo::roles::{Content, Querys},
    services::DynService,
    var::SOURCE_SYSTEM,
};

pub struct RolesRouter;

impl RolesRouter {
    pub fn new_router() -> Router {
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
    }

    async fn create(
        header: Header,
        Extension(srv): Extension<DynService>,
        Valid(Json(mut input)): Valid<Json<Content>>,
    ) -> Result<(StatusCode, Json<models::ID>)> {
        input.account_id = header.account_id;
        input.user_id = header.user_id;
        info!("list query {:#?}", input);
        let id = srv.role().create(&input).await?;
        Ok((StatusCode::CREATED, id.into()))
    }

    async fn list(
        header: Header,
        Valid(mut filter): Valid<Querys>,
        Extension(srv): Extension<DynService>,
    ) -> Result<Json<models::List<Role>>> {
        if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
            filter.account_id = Some(header.account_id);
        }
        filter.pagination.check();
        let list = srv.role().list(&filter).await?;
        Ok(list.into())
    }

    async fn get(
        header: Header,
        Path(id): Path<String>,
        Valid(mut filter): Valid<Querys>,
        Extension(srv): Extension<DynService>,
    ) -> Result<Json<RoleBindings>> {
        filter.pagination.check();
        if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
            filter.account_id = Some(header.account_id);
        }
        let resp = srv.role().get(&id, &filter).await?;
        Ok(resp.into())
    }

    async fn delete(
        header: Header,
        Path(id): Path<String>,
        Extension(srv): Extension<DynService>,
    ) -> Result<StatusCode> {
        let mut account_id = None;
        if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
            account_id = Some(header.account_id);
        }
        srv.role().delete(&id, account_id).await?;
        Ok(StatusCode::NO_CONTENT)
    }

    async fn put(
        header: Header,
        Path(id): Path<String>,
        Extension(srv): Extension<DynService>,
        Valid(Json(mut content)): Valid<Json<Content>>,
    ) -> Result<StatusCode> {
        info!("list query {:#?} {:#?}", content, header);
        content.account_id = header.account_id;
        content.user_id = header.user_id;
        srv.role().put(&id, &content).await?;
        Ok(StatusCode::NO_CONTENT)
    }

    async fn add_user(
        header: Header,
        Extension(srv): Extension<DynService>,
        Path((id, user_id)): Path<(String, String)>,
    ) -> Result<StatusCode> {
        info!("list query {:#?} {}", id, user_id);
        srv.role()
            .add_user(&id, &header.account_id, &user_id)
            .await?;
        Ok(StatusCode::NO_CONTENT)
    }

    async fn delete_user(
        header: Header,
        Extension(srv): Extension<DynService>,
        Path((id, user_id)): Path<(String, String)>,
    ) -> Result<StatusCode> {
        info!("list query {:#?} {} {}", header.account_id, id, user_id);
        srv.role().delete_user(&id, &user_id).await?;
        Ok(StatusCode::NO_CONTENT)
    }

    async fn add_policy(
        header: Header,
        Extension(srv): Extension<DynService>,
        Path((id, policy_id)): Path<(String, String)>,
    ) -> Result<StatusCode> {
        info!("list query {:#?} {}", id, policy_id);
        srv.role()
            .add_policy(&id, &header.account_id, &policy_id)
            .await?;
        Ok(StatusCode::NO_CONTENT)
    }

    async fn delete_policy(
        header: Header,
        Extension(srv): Extension<DynService>,
        Path((id, policy_id)): Path<(String, String)>,
    ) -> Result<StatusCode> {
        info!("list query {:#?} {} {}", header.account_id, id, policy_id);
        srv.role().delete_policy(&id, &policy_id).await?;
        Ok(StatusCode::NO_CONTENT)
    }
}
