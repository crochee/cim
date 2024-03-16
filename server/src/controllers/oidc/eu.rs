use askama::Template;
use axum::{
    extract::Path,
    response::{IntoResponse, Response},
    routing::get,
    Form, Router,
};
use chrono::Utc;
use http::{header, HeaderMap, StatusCode, Uri};
use slo::{errors, HtmlTemplate, Result};

use crate::{
    services::{
        authentication::connect,
        oidc::{
            self,
            auth::{AuthRequest, ReqHmac},
            get_connector, open_connector, parse_auth_request,
            password::{password_login, LoginData},
        },
    },
    valid::Valid,
    AppState,
};

pub fn new_router(state: AppState) -> Router {
    Router::new()
        .route("/auth/:connector_id", get(auth_html))
        .route("/auth/:connector_id/login", get(login_html).post(login))
        .route("/approval", get(approval_html).post(post_approval))
        .with_state(state)
}

const EXPIRATION: i64 = 3600;

async fn auth_html(
    app: AppState,
    Path(connector_id): Path<String>,
    Valid(auth_request): Valid<AuthRequest>,
) -> Response {
    let mut auth_req =
        match parse_auth_request(&app.store.client, &auth_request).await {
            Ok(v) => v,
            Err(err) => {
                return redirect(&auth_request, err);
            }
        };
    tracing::debug!("{:?}", auth_req);
    let connector =
        match get_connector(&app.store.connector, &connector_id).await {
            Ok(v) => v,
            Err(err) => return redirect(&auth_request, err),
        };
    let connnector_impl = match open_connector(&connector) {
        Ok(c) => c,
        Err(err) => {
            return HtmlTemplate(Password {
                post_url: format!(
                    "/login/{}?{}",
                    connector_id,
                    serde_urlencoded::to_string(auth_request)
                        .map_err(errors::any)
                        .unwrap()
                ),
                username: "".to_string(),
                username_prompt: format!("Enter your {}", connector.name),
                invalid: err.to_string(),
            })
            .into_response();
        }
    };
    auth_req.connector_id = connector_id.clone();
    auth_req.expires_in = Utc::now().timestamp() + EXPIRATION;

    let scopes = connect::parse_scopes(&auth_req.scopes);
    let redirect_uri = match connnector_impl {
        oidc::Connector::Password(_) => {
            format!(
                "/auth/{}/login?state={}&back={}",
                connector_id,
                auth_req.id,
                auth_request.back.unwrap()
            )
        }
        oidc::Connector::Callback(cc) => cc
            .login_url(&scopes, "/callback", &auth_req.id)
            .await
            .unwrap(),
        oidc::Connector::Saml(_) => "".to_string(),
    };

    let mut headers = HeaderMap::new();
    headers.insert(header::LOCATION, redirect_uri.parse().unwrap());
    (StatusCode::FOUND, headers).into_response()
}

fn redirect<E: ToString>(auth_request: &AuthRequest, err: E) -> Response {
    let mut redirect_uri = auth_request.redirect_uri.clone();
    if auth_request
        .redirect_uri
        .parse::<Uri>()
        .unwrap()
        .query()
        .is_none()
    {
        redirect_uri.push_str("?err=");
        redirect_uri.push_str(&err.to_string());
    } else {
        redirect_uri.push_str("&err=");
        redirect_uri.push_str(&err.to_string());
    };

    let mut headers = HeaderMap::new();
    headers.insert(header::LOCATION, redirect_uri.parse().unwrap());
    (StatusCode::SEE_OTHER, headers).into_response()
}

#[derive(Template)]
#[template(path = "password.html")]
pub struct Password {
    pub post_url: String,
    pub username: String,
    pub username_prompt: String,
    pub invalid: String,
}

async fn login_html(
    app: AppState,
    Path(connector_id): Path<String>,
    Valid(auth_request): Valid<AuthRequest>,
) -> Result<HtmlTemplate<Password>> {
    let connector = get_connector(&app.store.connector, &connector_id).await?;
    Ok(HtmlTemplate(Password {
        post_url: format!(
            "/login/{}?{}",
            connector_id,
            serde_urlencoded::to_string(auth_request).map_err(errors::any)?
        ),
        username: "".to_string(),
        username_prompt: format!("Enter your {}", connector.name),
        invalid: "".to_string(),
    }))
}

async fn login(
    app: AppState,
    Path(connector_id): Path<String>,
    Valid(mut auth_request): Valid<AuthRequest>,
    Valid(Form(login_data)): Valid<Form<LoginData>>,
) -> Response {
    let auth_req =
        match parse_auth_request(&app.store.client, &auth_request).await {
            Ok(v) => v,
            Err(err) => {
                return redirect(&auth_request, err);
            }
        };
    tracing::debug!("{:?}", auth_req);

    // 登录成功则跳转，否则返回登录页面
    match password_login(&app.store.client, &mut auth_request, &login_data)
        .await
    {
        Ok(v) => {
            let mut headers = HeaderMap::new();
            let redirect_uri = match v.parse().map_err(errors::any) {
                Ok(value) => value,
                Err(err) => {
                    return HtmlTemplate(Password {
                        post_url: format!(
                            "/login/{}?{}",
                            connector_id,
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
            post_url: format!(
                "/login/{}?{}",
                connector_id,
                serde_urlencoded::to_string(auth_request).unwrap()
            ),
            username: login_data.login.clone(),
            username_prompt: "Enter your username".to_string(),
            invalid: err.to_string(),
        })
        .into_response(),
    }
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
        client: "cim".to_string(),
        req: req_hmac.req,
        hmac: req_hmac.hmac,
        approval: req_hmac.approval,
    }))
}

async fn post_approval(
    _app: AppState,
    Valid(Form(req_hmac)): Valid<Form<ReqHmac>>,
) -> Response {
    tracing::debug!("{:?}", req_hmac);
    if let Some(approval) = &req_hmac.approval {
        if approval.eq("approve") {
            let mut headers = HeaderMap::new();
            headers.insert(header::LOCATION, "/".parse().unwrap());
            return (StatusCode::SEE_OTHER, headers).into_response();
        }
    }
    HtmlTemplate(Approval {
        issuer: "Cim".to_string(),
        scopes: vec![
            "openid".to_string(),
            "profile".to_string(),
            "email".to_string(),
        ],
        client: "cim".to_string(),
        req: req_hmac.req,
        hmac: req_hmac.hmac,
        approval: Some("approval".to_string()),
    })
    .into_response()
}
