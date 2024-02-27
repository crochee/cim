use axum::{routing::post, Json, Router};

use http::StatusCode;
use pim::Request;
use tracing::info;

use slo::Result;

use crate::{services::authorization, valid::Valid, AppState};

pub fn new_router(state: AppState) -> Router {
    Router::new()
        .route("/authorize", post(authorize))
        .with_state(state)
}

async fn authorize(
    app: AppState,
    Valid(Json(input)): Valid<Json<Request>>,
) -> Result<StatusCode> {
    info!("list query {:#?}", input);
    authorization::authorize(&app.store.policy, &app.matcher, &input).await?;
    Ok(StatusCode::NO_CONTENT)
}
