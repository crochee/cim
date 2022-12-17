use axum::{
    extract::Path,
    routing::{delete, get, post, put},
    Extension, Json, Router,
};
use cim_core::Result;
use http::StatusCode;
use tracing::info;

use crate::{
    models::{self, policy::Policy},
    repositories::policies::{Content, Querys},
    services::policies::DynPoliciesService,
    valid::{Header, Valid},
    var::SOURCE_SYSTEM,
    ServiceRegister,
};

pub struct PoliciesRouter;

impl PoliciesRouter {
    pub fn new_router(service_register: ServiceRegister) -> Router {
        Router::new()
            .route("/policies", post(Self::create))
            .route("/policies", get(Self::list))
            .route("/policies/:id", get(Self::get))
            .route("/policies/:id", delete(Self::delete))
            .route("/policies/:id", put(Self::put))
            .layer(Extension(service_register.policies_service))
    }

    async fn create(
        header: Header,
        Extension(policies_service): Extension<DynPoliciesService>,
        Valid(Json(mut content)): Valid<Json<Content>>,
    ) -> Result<(StatusCode, Json<models::ID>)> {
        info!("list query {:#?} {:#?}", content, header);
        if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
            content.account_id = Some(header.account_id);
            content.user_id = Some(header.user_id);
        }
        let id = policies_service.create(&content).await?;
        Ok((StatusCode::CREATED, id.into()))
    }

    async fn list(
        header: Header,
        Valid(mut filter): Valid<Querys>,
        Extension(policies_service): Extension<DynPoliciesService>,
    ) -> Result<Json<models::List<Policy>>> {
        if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
            filter.account_id = Some(header.account_id);
        }
        filter.pagination.check();
        let list = policies_service.list(&filter).await?;
        Ok(list.into())
    }

    async fn get(
        header: Header,
        Path(id): Path<String>,
        Extension(policies_service): Extension<DynPoliciesService>,
    ) -> Result<Json<Policy>> {
        let mut account_id = None;
        if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
            account_id = Some(header.account_id);
        }
        let resp = policies_service.get(&id, account_id).await?;
        Ok(resp.into())
    }

    async fn delete(
        header: Header,
        Path(id): Path<String>,
        Extension(policies_service): Extension<DynPoliciesService>,
    ) -> Result<StatusCode> {
        let mut account_id = None;
        if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
            account_id = Some(header.account_id);
        }
        policies_service.delete(&id, account_id).await?;
        Ok(StatusCode::NO_CONTENT)
    }

    async fn put(
        header: Header,
        Path(id): Path<String>,
        Extension(policies_service): Extension<DynPoliciesService>,
        Valid(Json(mut content)): Valid<Json<Content>>,
    ) -> Result<StatusCode> {
        info!("list query {:#?} {:#?}", content, header);
        if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
            content.account_id = Some(header.account_id);
            content.user_id = Some(header.user_id);
        }
        policies_service.put(&id, &content).await?;
        Ok(StatusCode::NO_CONTENT)
    }
}
