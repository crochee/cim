use axum::{extract::Path, routing::get, Extension, Json, Router};

use cim_core::Result;
use http::StatusCode;
use tracing::info;

use crate::{
    models::{self, user::User},
    pkg::valid::{Header, Valid},
    repo::users::{Content, Querys},
    services::DynService,
    var::SOURCE_SYSTEM,
};

pub struct UsersRouter;

impl UsersRouter {
    pub fn new_router() -> Router {
        Router::new()
            .route("/users", get(Self::list).post(Self::create))
            .route(
                "/users/:id",
                get(Self::get).delete(Self::delete).put(Self::put),
            )
    }

    async fn create(
        Extension(srv): Extension<DynService>,
        Valid(Json(input)): Valid<Json<Content>>,
    ) -> Result<(StatusCode, Json<models::ID>)> {
        info!("list query {:#?}", input);
        let id = srv.user().create(&input).await?;
        Ok((StatusCode::CREATED, id.into()))
    }

    async fn list(
        header: Header,
        Valid(mut filter): Valid<Querys>,
        Extension(srv): Extension<DynService>,
    ) -> Result<Json<models::List<User>>> {
        if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
            filter.account_id = Some(header.account_id);
        }
        filter.pagination.check();
        let list = srv.user().list(&filter).await?;
        Ok(list.into())
    }

    async fn get(
        header: Header,
        Path(id): Path<String>,
        Extension(srv): Extension<DynService>,
    ) -> Result<Json<User>> {
        let mut account_id = None;
        if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
            account_id = Some(header.account_id);
        }
        let resp = srv.user().get(&id, account_id).await?;
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
        srv.user().delete(&id, account_id).await?;
        Ok(StatusCode::NO_CONTENT)
    }

    async fn put(
        header: Header,
        Path(id): Path<String>,
        Extension(srv): Extension<DynService>,
        Valid(Json(mut content)): Valid<Json<Content>>,
    ) -> Result<StatusCode> {
        info!("list query {:#?} {:#?}", content, header);
        if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
            content.account_id = Some(header.account_id);
        }
        srv.user().put(&id, &content).await?;
        Ok(StatusCode::NO_CONTENT)
    }
}
