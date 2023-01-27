use axum::{
    extract::Path,
    routing::{delete, get, post, put},
    Extension, Json, Router,
};

use cim_core::Result;
use http::StatusCode;
use tracing::info;

use crate::{
    models::{self, role::Role},
    pkg::valid::{Header, Valid},
    repo::roles::{Content, Querys},
    services::roles::DynRolesService,
    var::SOURCE_SYSTEM,
    ServiceRegister,
};

pub struct RolesRouter;

impl RolesRouter {
    pub fn new_router(service_register: ServiceRegister) -> Router {
        Router::new()
            .route("/roles", post(Self::create))
            .route("/roles", get(Self::list))
            .route("/roles/:id", get(Self::get))
            .route("/roles/:id", delete(Self::delete))
            .route("/roles/:id", put(Self::put))
            .layer(axum::Extension(service_register.roles_service))
    }

    async fn create(
        header: Header,
        Extension(roles_service): Extension<DynRolesService>,
        Valid(Json(mut input)): Valid<Json<Content>>,
    ) -> Result<(StatusCode, Json<models::ID>)> {
        input.account_id = header.account_id;
        input.user_id = header.user_id;
        info!("list query {:#?}", input);
        let id = roles_service.create(&input).await?;
        Ok((StatusCode::CREATED, id.into()))
    }

    async fn list(
        header: Header,
        Valid(mut filter): Valid<Querys>,
        Extension(roles_service): Extension<DynRolesService>,
    ) -> Result<Json<models::List<Role>>> {
        if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
            filter.account_id = Some(header.account_id);
        }
        filter.pagination.check();
        let list = roles_service.list(&filter).await?;
        Ok(list.into())
    }

    async fn get(
        header: Header,
        Path(id): Path<String>,
        Extension(roles_service): Extension<DynRolesService>,
    ) -> Result<Json<Role>> {
        let mut account_id = None;
        if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
            account_id = Some(header.account_id);
        }
        let resp = roles_service.get(&id, account_id).await?;
        Ok(resp.into())
    }

    async fn delete(
        header: Header,
        Path(id): Path<String>,
        Extension(roles_service): Extension<DynRolesService>,
    ) -> Result<StatusCode> {
        let mut account_id = None;
        if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
            account_id = Some(header.account_id);
        }
        roles_service.delete(&id, account_id).await?;
        Ok(StatusCode::NO_CONTENT)
    }

    async fn put(
        header: Header,
        Path(id): Path<String>,
        Extension(roles_service): Extension<DynRolesService>,
        Valid(Json(mut content)): Valid<Json<Content>>,
    ) -> Result<StatusCode> {
        info!("list query {:#?} {:#?}", content, header);
        content.account_id = header.account_id;
        content.user_id = header.user_id;
        roles_service.put(&id, &content).await?;
        Ok(StatusCode::NO_CONTENT)
    }
}
