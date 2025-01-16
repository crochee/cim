use std::collections::HashMap;

use askama::Template;
use axum::{
    extract::{Path, Query, Request},
    response::Redirect,
    routing::{get, post},
    Json, Router,
};
use http::{StatusCode, Uri};

use cim_slo::{errors, HtmlTemplate, Result};
use tracing::info;

use crate::{
    services::{
        authorization,
        oidc::{
            auth, auth_page_callback, get_auth_request, get_connector,
            parse_auth_request, redirect_auth_page,
        },
    },
    valid::Valid,
    AppState,
};

pub fn new_router(state: AppState) -> Router {
    Router::new()
        .route("/connectors/{connector_id}", get(connector_handle))
        .route("/callback", get(callback_handle))
        // cim impl callback
        .route("/login", get(password_login_html))
        // example redirect_uri api
        .route("/redirect", get(redirect_callback))
        .route("/autth", post(authorize))
        .with_state(state)
}

async fn authorize(
    app: AppState,
    Valid(Json(input)): Valid<Json<cim_pim::Request>>,
) -> Result<StatusCode> {
    info!("list query {:#?}", input);
    authorization::authorize(&app.store.statement, &app.matcher, &input)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn redirect_callback(
    Query(hm): Query<HashMap<String, String>>,
) -> Json<HashMap<String, String>> {
    Json(hm)
}

/// redirect to user auth page,example password_login_html or server auth page
async fn connector_handle(
    app: AppState,
    Path(connector_id): Path<String>,
    Valid(auth_request): Valid<auth::AuthRequest>,
) -> Result<Redirect> {
    let mut auth_req =
        parse_auth_request(&app.store.client, &auth_request).await?;
    let connector = get_connector(&app.store.connector, &connector_id).await?;

    let redirect_uri = redirect_auth_page(
        &app.store.auth_request,
        &connector,
        &app.store.user,
        &connector_id,
        &mut auth_req,
        app.config.expiration,
    )
    .await?;
    Ok(Redirect::to(&redirect_uri))
}

/// auth handle finish login,should request by server which auth_handle redirect
async fn callback_handle(
    app: AppState,
    Valid(state): Valid<auth::AuthRequestState>,
    req: Request,
) -> Result<Redirect> {
    let redirect_uri = auth_page_callback(
        &app.store.auth_request,
        &app.store.user,
        &app.store.auth_code,
        &app.store.connector,
        &app.store.offline_session,
        &app.access_token,
        &state,
        req,
    )
    .await?;
    Ok(Redirect::to(&redirect_uri))
}

#[derive(Template, Default)]
#[template(path = "password.html")]
pub struct Password {
    pub post_url: String,
    pub scopes: Vec<String>,
    pub username_prompt: String,
}

async fn password_login_html(
    app: AppState,
    Valid(state): Valid<auth::AuthRequestState>,
) -> Result<HtmlTemplate<Password>> {
    let auth_req =
        get_auth_request(&app.store.auth_request, &state.state).await?;

    let mut u = state.callback.unwrap_or("/callback".to_string());
    let url = u.parse::<Uri>().map_err(|err| errors::bad_request(&err))?;
    if url.query().is_none() {
        u.push('?');
    } else {
        u.push('&');
    }
    u.push_str("state=");
    u.push_str(&state.state);

    Ok(HtmlTemplate(Password {
        post_url: u,
        scopes: auth_req.scopes,
        username_prompt: "Enter your username".to_string(),
    }))
}
