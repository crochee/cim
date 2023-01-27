use axum::{
    routing::{get, post},
    Extension, Json, Router,
};

use cim_core::Result;
use http::StatusCode;
use serde::Deserialize;
use tracing::info;

use crate::{
    models::{self, rolebinding::RoleBinding},
    pkg::valid::{Header, Valid},
    repo::rolebindings::{Content, Opts, Querys},
    services::rolebindings::DynRoleBindingsService,
    ServiceRegister,
};

pub struct RoleBindingsRouter;

impl RoleBindingsRouter {
    pub fn new_router(service_register: ServiceRegister) -> Router {
        Router::new()
            .route("/rolebindings/batch", post(Self::batch))
            .route("/rolebindings", get(Self::list))
            .layer(axum::Extension(service_register.rolebindings_service))
    }

    async fn batch(
        header: Header,
        Extension(rolebindings_service): Extension<DynRoleBindingsService>,
        Json(input): Json<BachBody>,
    ) -> Result<StatusCode> {
        info!("list query {:#?}", input);
        match input {
            BachBody::Create(v) => {
                rolebindings_service.create(header.account_id, &v).await?
            }
            BachBody::Delete(v) => rolebindings_service.delete(&v).await?,
        }
        Ok(StatusCode::NO_CONTENT)
    }

    async fn list(
        _header: Header,
        Valid(mut filter): Valid<Querys>,
        Extension(rolebindings_service): Extension<DynRoleBindingsService>,
    ) -> Result<Json<models::List<RoleBinding>>> {
        filter.pagination.check();
        let list = rolebindings_service.list(&filter).await?;
        Ok(list.into())
    }
}

#[derive(Debug, Deserialize)]
pub enum BachBody {
    Create(Content),
    Delete(Opts),
}
