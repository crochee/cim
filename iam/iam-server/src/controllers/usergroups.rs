use axum::{
    extract::Path,
    routing::{delete, get, post, put},
    Extension, Json, Router,
};

use cim_core::Result;
use http::StatusCode;
use tracing::info;

use crate::{
    models::{self, usergroup::UserGroup},
    pkg::valid::{Header, Valid},
    repo::usergroups::{Content, Querys},
    services::usergroups::DynUserGroupsService,
    var::SOURCE_SYSTEM,
    ServiceRegister,
};

pub struct UserGroupsRouter;

impl UserGroupsRouter {
    pub fn new_router(service_register: ServiceRegister) -> Router {
        Router::new()
            .route("/groups", post(Self::create))
            .route("/groups", get(Self::list))
            .route("/groups/:id", get(Self::get))
            .route("/groups/:id", delete(Self::delete))
            .route("/groups/:id", put(Self::put))
            .layer(axum::Extension(service_register.usergroups_service))
    }

    async fn create(
        header: Header,
        Extension(user_groups_service): Extension<DynUserGroupsService>,
        Valid(Json(mut input)): Valid<Json<Content>>,
    ) -> Result<(StatusCode, Json<models::ID>)> {
        input.account_id = header.account_id;
        input.user_id = header.user_id;
        info!("list query {:#?}", input);
        let id = user_groups_service.create(&input).await?;
        Ok((StatusCode::CREATED, id.into()))
    }

    async fn list(
        header: Header,
        Valid(mut filter): Valid<Querys>,
        Extension(user_groups_service): Extension<DynUserGroupsService>,
    ) -> Result<Json<models::List<UserGroup>>> {
        if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
            filter.account_id = Some(header.account_id);
        }
        filter.pagination.check();
        let list = user_groups_service.list(&filter).await?;
        Ok(list.into())
    }

    async fn get(
        header: Header,
        Path(id): Path<String>,
        Extension(user_groups_service): Extension<DynUserGroupsService>,
    ) -> Result<Json<UserGroup>> {
        let mut account_id = None;
        if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
            account_id = Some(header.account_id);
        }
        let resp = user_groups_service.get(&id, account_id).await?;
        Ok(resp.into())
    }

    async fn delete(
        header: Header,
        Path(id): Path<String>,
        Extension(user_groups_service): Extension<DynUserGroupsService>,
    ) -> Result<StatusCode> {
        let mut account_id = None;
        if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
            account_id = Some(header.account_id);
        }
        user_groups_service.delete(&id, account_id).await?;
        Ok(StatusCode::NO_CONTENT)
    }

    async fn put(
        header: Header,
        Path(id): Path<String>,
        Extension(user_groups_service): Extension<DynUserGroupsService>,
        Valid(Json(mut content)): Valid<Json<Content>>,
    ) -> Result<StatusCode> {
        info!("list query {:#?} {:#?}", content, header);
        content.account_id = header.account_id;
        content.user_id = header.user_id;
        user_groups_service.put(&id, &content).await?;
        Ok(StatusCode::NO_CONTENT)
    }
}
