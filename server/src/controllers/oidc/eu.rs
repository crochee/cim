use askama::Template;
use axum::{
    response::{IntoResponse, Response},
    routing::get,
    Form, Router,
};
use http::{header, HeaderMap, StatusCode};
use slo::{errors, HtmlTemplate, Result};

use crate::{
    services::oidc::{
        auth::{AuthRequest, ReqHmac},
        password::{password_login, LoginData},
    },
    valid::Valid,
    AppState,
};

pub fn new_router(state: AppState) -> Router {
    Router::new()
        .route("/login/:id", get(login_html))
        .route("/login", get(login_html).post(login))
        .route("/approval", get(approval_html))
        .with_state(state)
}

#[derive(Template)]
#[template(path = "password.html")]
pub struct Password {
    pub issuer: String,
    pub post_url: String,
    pub username: String,
    pub username_prompt: String,
    pub invalid: String,
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
        invalid: "".to_string(),
    }))
}

#[derive(Template)]
#[template(path = "approval.html")]
pub struct Approval {
    pub issuer: String,
    pub scopes: Vec<String>,
    pub client: String,
    pub auth_req_id: String,
}

async fn approval_html(
    _app: AppState,
    Valid(req_hmac): Valid<ReqHmac>,
) -> Result<HtmlTemplate<Approval>> {
    Ok(HtmlTemplate(Approval {
        issuer: "Cim".to_string(),
        scopes: vec![
            "openid".to_string(),
            "profile".to_string(),
            "email".to_string(),
        ],
        client: req_hmac.req,
        auth_req_id: req_hmac.hmac,
    }))
}

async fn login(
    _app: AppState,
    Valid(auth_request): Valid<AuthRequest>,
    Valid(Form(login_data)): Valid<Form<LoginData>>,
) -> Response {
    // 登录成功则跳转，否则返回登录页面
    match password_login(&auth_request, &login_data).await {
        Ok(v) => {
            let mut headers = HeaderMap::new();
            let redirect_uri = match v.parse().map_err(errors::any) {
                Ok(value) => value,
                Err(err) => {
                    return HtmlTemplate(Password {
                        issuer: "Cim".to_string(),
                        post_url: format!(
                            "/login?{}",
                            serde_urlencoded::to_string(auth_request).unwrap()
                        ),
                        username: login_data.login.clone(),
                        username_prompt: "Enter your username".to_string(),
                        invalid: err.to_string(),
                    })
                    .into_response()
                }
            };
            headers.insert(header::LOCATION, redirect_uri);
            (StatusCode::SEE_OTHER, headers).into_response()
        }
        Err(err) => HtmlTemplate(Password {
            issuer: "Cim".to_string(),
            post_url: format!(
                "/login?{}",
                serde_urlencoded::to_string(auth_request).unwrap()
            ),
            username: login_data.login.clone(),
            username_prompt: "Enter your username".to_string(),
            invalid: err.to_string(),
        })
        .into_response(),
    }
}
