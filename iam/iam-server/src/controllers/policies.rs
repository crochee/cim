use std::collections::HashMap;

use axum::{
    extract::{Query, RawQuery},
    routing::{delete, get, patch, post, put},
    Extension, Json, Router,
};
use cim_core::{Error, Result};
use tracing::info;

use crate::{
    models,
    services::policies::{self, DynPoliciesService, Policy},
    valid::Valid,
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

    async fn create() -> Result<()> {
        Ok(())
    }

    async fn list(
        Valid(Query(filter)): Valid<Query<policies::Filter>>,
        Extension(policies_service): Extension<DynPoliciesService>,
    ) -> Result<Json<models::List<Policy>>> {
        info!("list query {:#?}", filter);
        Ok(models::List {
            data: vec![],
            limit: 0,
            offset: 0,
            total: 0,
        }
        .into())
        // let list = policies_service.list(&filter).await?;
        // Ok(list.into())
    }

    async fn get() -> Result<()> {
        Ok(())
    }

    async fn delete() -> Result<()> {
        Ok(())
    }

    async fn put() -> Result<()> {
        Ok(())
    }
}
