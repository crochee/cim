use axum::{extract::Path, routing::get, Extension, Json, Router};
use cim_core::Result;
use http::StatusCode;
use tracing::info;

use crate::{
    models::{self, policy::Policy},
    pkg::valid::{Header, Valid},
    repo::policies::{Content, Querys},
    services::DynService,
    var::SOURCE_SYSTEM,
};

pub struct PoliciesRouter;

impl PoliciesRouter {
    pub fn new_router() -> Router {
        Router::new()
            .route("/policies", get(Self::list).post(Self::create))
            .route(
                "/policies/:id",
                get(Self::get).delete(Self::delete).put(Self::put),
            )
    }

    async fn create(
        header: Header,
        Extension(srv): Extension<DynService>,
        Valid(Json(mut content)): Valid<Json<Content>>,
    ) -> Result<(StatusCode, Json<models::ID>)> {
        info!("list query {:#?} {:#?}", content, header);
        if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
            content.account_id = Some(header.account_id);
            content.user_id = Some(header.user_id);
        }
        let id = srv.policy().create(&content).await?;
        Ok((StatusCode::CREATED, id.into()))
    }

    async fn list(
        header: Header,
        Valid(mut filter): Valid<Querys>,
        Extension(srv): Extension<DynService>,
    ) -> Result<Json<models::List<Policy>>> {
        if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
            filter.account_id = Some(header.account_id);
        }
        filter.pagination.check();
        let list = srv.policy().list(&filter).await?;
        Ok(list.into())
    }

    async fn get(
        header: Header,
        Path(id): Path<String>,
        Extension(srv): Extension<DynService>,
    ) -> Result<Json<Policy>> {
        let mut account_id = None;
        if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
            account_id = Some(header.account_id);
        }
        let resp = srv.policy().get(&id, account_id).await?;
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
        srv.policy().delete(&id, account_id).await?;
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
            content.user_id = Some(header.user_id);
        }
        srv.policy().put(&id, &content).await?;
        Ok(StatusCode::NO_CONTENT)
    }
}
