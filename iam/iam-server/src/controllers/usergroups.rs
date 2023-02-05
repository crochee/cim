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
        usergroup::{UserGroup, UserGroupBindings},
    },
    pkg::valid::{Header, Valid},
    repo::usergroups::{Content, Querys},
    services::DynService,
    var::SOURCE_SYSTEM,
};

pub struct UserGroupsRouter;

impl UserGroupsRouter {
    pub fn new_router() -> Router {
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
    }

    async fn create(
        header: Header,
        Extension(srv): Extension<DynService>,
        Valid(Json(mut input)): Valid<Json<Content>>,
    ) -> Result<(StatusCode, Json<models::ID>)> {
        input.account_id = header.account_id;
        input.user_id = header.user_id;
        info!("list query {:#?}", input);
        let id = srv.user_group().create(&input).await?;
        Ok((StatusCode::CREATED, id.into()))
    }

    async fn list(
        header: Header,
        Valid(mut filter): Valid<Querys>,
        Extension(srv): Extension<DynService>,
    ) -> Result<Json<models::List<UserGroup>>> {
        if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
            filter.account_id = Some(header.account_id);
        }
        filter.pagination.check();
        let list = srv.user_group().list(&filter).await?;
        Ok(list.into())
    }

    async fn get(
        header: Header,
        Path(id): Path<String>,
        Valid(mut filter): Valid<Querys>,
        Extension(srv): Extension<DynService>,
    ) -> Result<Json<UserGroupBindings>> {
        filter.pagination.check();
        if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
            filter.account_id = Some(header.account_id);
        }
        let resp = srv.user_group().get(&id, &filter).await?;
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
        srv.user_group().delete(&id, account_id).await?;
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
        srv.user_group().put(&id, &content).await?;
        Ok(StatusCode::NO_CONTENT)
    }

    async fn add_user(
        header: Header,
        Extension(srv): Extension<DynService>,
        Path((id, user_id)): Path<(String, String)>,
    ) -> Result<StatusCode> {
        info!("list query {:#?} {}", id, user_id);
        srv.user_group()
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
        srv.user_group().delete_user(&id, &user_id).await?;
        Ok(StatusCode::NO_CONTENT)
    }

    async fn add_role(
        header: Header,
        Extension(srv): Extension<DynService>,
        Path((id, role_id)): Path<(String, String)>,
    ) -> Result<StatusCode> {
        info!("list query {:#?} {}", id, role_id);
        srv.user_group()
            .add_role(&id, &header.account_id, &role_id)
            .await?;
        Ok(StatusCode::NO_CONTENT)
    }

    async fn delete_role(
        header: Header,
        Extension(srv): Extension<DynService>,
        Path((id, role_id)): Path<(String, String)>,
    ) -> Result<StatusCode> {
        info!("list query {:#?} {} {}", header.account_id, id, role_id);
        srv.user_group().delete_role(&id, &role_id).await?;
        Ok(StatusCode::NO_CONTENT)
    }
}
