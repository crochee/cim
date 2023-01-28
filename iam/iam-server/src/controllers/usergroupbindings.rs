use axum::{
    routing::{get, post},
    Extension, Json, Router,
};

use cim_core::Result;
use http::StatusCode;
use serde::Deserialize;
use tracing::info;

use crate::{
    models::{self, usergroupbinding::UserGroupBinding},
    pkg::valid::{Header, Valid},
    repo::usergroupbindings::{Content, Opts, Querys},
    services::usergroupbindings::DynUserGroupBindingsService,
    ServiceRegister,
};

pub struct UserGroupBindingsRouter;

impl UserGroupBindingsRouter {
    pub fn new_router(service_register: ServiceRegister) -> Router {
        Router::new()
            .route("/groupbindings/batch", post(Self::batch))
            .route("/groupbindings", get(Self::list))
            .layer(axum::Extension(service_register.usergroupbindings_service))
    }

    async fn batch(
        header: Header,
        Extension(usergroupbindings_service): Extension<
            DynUserGroupBindingsService,
        >,
        Json(input): Json<BachBody>,
    ) -> Result<StatusCode> {
        info!("list query {:#?}", input);
        match input {
            BachBody::Create(v) => {
                usergroupbindings_service
                    .create(header.account_id, &v)
                    .await?
            }
            BachBody::Delete(v) => usergroupbindings_service.delete(&v).await?,
        }
        Ok(StatusCode::NO_CONTENT)
    }

    async fn list(
        _header: Header,
        Valid(mut filter): Valid<Querys>,
        Extension(usergroupbindings_service): Extension<
            DynUserGroupBindingsService,
        >,
    ) -> Result<Json<models::List<UserGroupBinding>>> {
        filter.pagination.check();
        let list = usergroupbindings_service.list(&filter).await?;
        Ok(list.into())
    }
}

#[derive(Debug, Deserialize)]
pub enum BachBody {
    Create(Content),
    Delete(Opts),
}
