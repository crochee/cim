use crate::Result;
use axum::{extract::Path, routing::get, Json, Router};
use http::StatusCode;
use tracing::info;

use crate::{
    models::{self, policy::Policy},
    pkg::valid::{Header, Valid},
    services::policies,
    store::{
        policies::{Content, Querys},
        Store,
    },
    var::SOURCE_SYSTEM,
    AppState,
};

pub struct PoliciesRouter;

impl PoliciesRouter {
    pub fn new_router(state: AppState) -> Router {
        Router::new()
            .route("/policies", get(Self::list).post(Self::create))
            .route(
                "/policies/:id",
                get(Self::get).delete(Self::delete).put(Self::put),
            )
            .with_state(state)
    }

    async fn create(
        header: Header,
        app: AppState,
        Valid(Json(mut content)): Valid<Json<Content>>,
    ) -> Result<(StatusCode, Json<models::ID>)> {
        info!("list query {:#?} {:#?}", content, header);
        if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
            content.account_id = Some(header.account_id);
            content.user_id = Some(header.user_id);
        }
        let id = app.store.create_policy(None, &content).await?;
        Ok((StatusCode::CREATED, id.into()))
    }

    async fn list(
        header: Header,
        app: AppState,
        Valid(mut filter): Valid<Querys>,
    ) -> Result<Json<models::List<Policy>>> {
        if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
            filter.account_id = Some(header.account_id);
        }
        filter.pagination.check();
        let list = app.store.list_policy(&filter).await?;
        Ok(list.into())
    }

    async fn get(
        header: Header,
        app: AppState,
        Path(id): Path<String>,
    ) -> Result<Json<Policy>> {
        let mut account_id = None;
        if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
            account_id = Some(header.account_id);
        }
        let resp = app.store.get_policy(&id, account_id).await?;
        Ok(resp.into())
    }

    async fn delete(
        header: Header,
        app: AppState,
        Path(id): Path<String>,
    ) -> Result<StatusCode> {
        let mut account_id = None;
        if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
            account_id = Some(header.account_id);
        }
        app.store.delete_policy(&id, account_id).await?;
        Ok(StatusCode::NO_CONTENT)
    }

    async fn put(
        header: Header,
        app: AppState,
        Path(id): Path<String>,
        Valid(Json(mut content)): Valid<Json<Content>>,
    ) -> Result<StatusCode> {
        info!("list query {:#?} {:#?}", content, header);
        if header.source.ne(&Some(SOURCE_SYSTEM.to_owned())) {
            content.account_id = Some(header.account_id);
            content.user_id = Some(header.user_id);
        }
        policies::put(&app, &id, &content).await?;
        Ok(StatusCode::NO_CONTENT)
    }
}
