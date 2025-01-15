use std::collections::HashMap;

use askama::Template;
use axum::{
    extract::{Path, Query, Request},
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Form, Json, Router,
};
use http::{header, HeaderMap, StatusCode, Uri};

use cim_slo::{errors, HtmlTemplate, Result};

use crate::{
    services::oidc::{
        auth, auth_page_callback, get_connector, parse_auth_request,
        redirect_auth_page, send_code, verify_auth_request,
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
        .route("/approval", get(approval_html).post(post_approval))
        // example redirect_uri api
        .route("/redirect", get(redirect_callback))
        .with_state(state)
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
    pub username_prompt: String,
}

async fn password_login_html(
    Valid(state): Valid<auth::AuthRequestState>,
) -> Result<HtmlTemplate<Password>> {
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
        username_prompt: "Enter your username".to_string(),
    }))
}

#[derive(Template)]
#[template(path = "approval.html")]
pub struct Approval {
    pub issuer: String,
    pub scopes: Vec<String>,
    pub client: String,
    pub req: String,
    pub hmac: String,
    pub approval: Option<String>,
}

async fn approval_html(
    app: AppState,
    Valid(req_hmac): Valid<auth::ReqHmac>,
) -> Result<HtmlTemplate<Approval>> {
    verify_auth_request(&app.store.auth_request, &req_hmac).await?;

    Ok(HtmlTemplate(Approval {
        issuer: "Cim".to_string(),
        scopes: vec![
            "openid".to_string(),
            "profile".to_string(),
            "email".to_string(),
        ],
        client: "cim".to_string(),
        req: req_hmac.req,
        hmac: req_hmac.hmac,
        approval: req_hmac.approval,
    }))
}

async fn post_approval(
    app: AppState,
    Valid(Form(req_hmac)): Valid<Form<auth::ReqHmac>>,
) -> Result<Response> {
    if let Some(approval) = &req_hmac.approval {
        if approval.eq("approve") {
            let auth_req =
                verify_auth_request(&app.store.auth_request, &req_hmac).await?;

            let url = send_code(
                &app.store.auth_request,
                &app.access_token,
                &app.store.auth_code,
                &auth_req,
            )
            .await?;
            let mut headers = HeaderMap::new();
            headers.insert(header::LOCATION, url.parse().unwrap());
            return Ok((StatusCode::SEE_OTHER, headers).into_response());
        }
    }
    Ok(HtmlTemplate(Approval {
        issuer: "Cim".to_string(),
        scopes: vec![
            "openid".to_string(),
            "profile".to_string(),
            "email".to_string(),
        ],
        client: "cim".to_string(),
        req: req_hmac.req,
        hmac: req_hmac.hmac,
        approval: req_hmac.approval,
    })
    .into_response())
}
