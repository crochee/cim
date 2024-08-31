use axum::{routing::post, Json, Router};

use http::StatusCode;
use tracing::info;

use cim_pim::Request;
use cim_slo::Result;

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
    authorization::authorize(&app.store.statement, &app.matcher, &input).await?;
    Ok(StatusCode::NO_CONTENT)
}
