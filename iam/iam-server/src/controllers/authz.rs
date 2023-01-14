use axum::{routing::post, Extension, Json, Router};

use cim_core::Result;
use http::StatusCode;
use tracing::info;

use crate::{
    models::req::Request, services::authorization::DynAuthorizer, valid::Valid,
    ServiceRegister,
};

pub struct AuthzRouter;

impl AuthzRouter {
    pub fn new_router(service_register: ServiceRegister) -> Router {
        Router::new()
            .route("/authz", post(Self::authorize))
            .layer(Extension(service_register.authorizer))
    }

    async fn authorize(
        Extension(authorizer): Extension<DynAuthorizer>,
        Valid(Json(input)): Valid<Json<Request>>,
    ) -> Result<StatusCode> {
        info!("list query {:#?}", input);
        authorizer.authorize(&input).await?;
        Ok(StatusCode::NO_CONTENT)
    }
}
