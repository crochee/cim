use axum::{
    extract::Path,
    routing::{delete, get, post, put},
    Extension, Json, Router,
};

use cim_core::Result;
use http::StatusCode;
use tracing::info;

use crate::{
    models::{self, user::User},
    pkg::valid::{Header, Valid},
    repo::users::{Content, Querys},
    services::users::DynUsersService,
    var::SOURCE_SYSTEM,
    ServiceRegister,
};

pub struct UsersRouter;

impl UsersRouter {
    pub fn new_router(service_register: ServiceRegister) -> Router {
        Router::new()
            .route("/users", post(Self::create))
            .route("/users", get(Self::list))
            .route("/users/:id", get(Self::get))
            .route("/users/:id", delete(Self::delete))
            .route("/users/:id", put(Self::put))
            .layer(axum::Extension(service_register.users_service))
    }

    async fn create(
        Extension(users_service): Extension<DynUsersService>,
        Valid(Json(input)): Valid<Json<Content>>,
    ) -> Result<(StatusCode, Json<models::ID>)> {
        info!("list query {:#?}", input);
        let id = users_service.create(&input).await?;
        Ok((StatusCode::CREATED, id.into()))
    }

    async fn list(
        header: Header,
        Valid(mut filter): Valid<Querys>,
        Extension(users_service): Extension<DynUsersService>,
    ) -> Result<Json<models::List<User>>> {
        if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
            filter.account_id = Some(header.account_id);
        }
        filter.pagination.check();
        let list = users_service.list(&filter).await?;
        Ok(list.into())
    }

    async fn get(
        header: Header,
        Path(id): Path<String>,
        Extension(users_service): Extension<DynUsersService>,
    ) -> Result<Json<User>> {
        let mut account_id = None;
        if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
            account_id = Some(header.account_id);
        }
        let resp = users_service.get(&id, account_id).await?;
        Ok(resp.into())
    }

    async fn delete(
        header: Header,
        Path(id): Path<String>,
        Extension(users_service): Extension<DynUsersService>,
    ) -> Result<StatusCode> {
        let mut account_id = None;
        if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
            account_id = Some(header.account_id);
        }
        users_service.delete(&id, account_id).await?;
        Ok(StatusCode::NO_CONTENT)
    }

    async fn put(
        header: Header,
        Path(id): Path<String>,
        Extension(users_service): Extension<DynUsersService>,
        Valid(Json(mut content)): Valid<Json<Content>>,
    ) -> Result<StatusCode> {
        info!("list query {:#?} {:#?}", content, header);
        if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
            content.account_id = Some(header.account_id);
        }
        users_service.put(&id, &content).await?;
        Ok(StatusCode::NO_CONTENT)
    }
}
