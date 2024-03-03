use askama::Template;
use axum::{routing::get, Form, Router};
use http::{header, HeaderMap, StatusCode};
use slo::{errors, HtmlTemplate, Result};

use crate::{
    services::oidc::{
        auth::AuthRequest,
        password::{password_login, LoginData},
    },
    valid::Valid,
    AppState,
};

pub fn new_router(state: AppState) -> Router {
    Router::new()
        .route("/login/:id", get(login_html))
        .route("/login", get(login_html).post(login))
        .with_state(state)
}

#[derive(Template)]
#[template(path = "password.html")]
pub struct Password {
    pub issuer: String,
    pub post_url: String,
    pub username: String,
    pub username_prompt: String,
    pub invalid: bool,
}

async fn login_html(
    _app: AppState,
    Valid(auth_request): Valid<AuthRequest>,
) -> Result<HtmlTemplate<Password>> {
    Ok(HtmlTemplate(Password {
        issuer: "Cim".to_string(),
        post_url: format!(
            "/login?{}",
            serde_urlencoded::to_string(auth_request).map_err(errors::any)?
        ),
        username: "".to_string(),
        username_prompt: "Enter your username".to_string(),
        invalid: false,
    }))
}

async fn login(
    _app: AppState,
    Valid(auth_request): Valid<AuthRequest>,
    Valid(Form(login_data)): Valid<Form<LoginData>>,
) -> Result<(StatusCode, HeaderMap)> {
    let redirect_uri = password_login(&auth_request, &login_data).await?;
    let mut headers = HeaderMap::new();
    headers
        .insert(header::LOCATION, redirect_uri.parse().map_err(errors::any)?);
    Ok((StatusCode::SEE_OTHER, headers))
}
