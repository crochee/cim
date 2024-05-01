use askama::Template;
use axum::{
    extract::Path,
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Form, Router,
};
use http::{header, HeaderMap, StatusCode, Uri};

use cim_slo::{errors, HtmlTemplate, Result};

use crate::{
    services::oidc::{
        self, auth, connect, get_connector, open_connector, parse_auth_request,
        password::{
            self, finalize_login, get_auth_request, send_code, LoginData,
        },
        run_connector,
    },
    valid::Valid,
    AppState,
};

pub fn new_router(state: AppState) -> Router {
    Router::new()
        .route("/auth/:connector_id", get(auth_html))
        .route(
            "/auth/:connector_id/login",
            get(password_login_html).post(password_login_handle),
        )
        .route("/approval", get(approval_html).post(post_approval))
        .with_state(state)
}

async fn auth_html(
    app: AppState,
    Path(connector_id): Path<String>,
    Valid(auth_request): Valid<auth::AuthRequest>,
) -> core::result::Result<Redirect, Redirect> {
    let mut auth_req = parse_auth_request(&app.store.client, &auth_request)
        .await
        .map_err(|err| redirect(&auth_request, err))?;
    let connector = get_connector(&app.store.connector, &connector_id)
        .await
        .map_err(|err| redirect(&auth_request, err))?;

    let redirect_uri = run_connector(
        &app.store.auth_request,
        &connector,
        &app.store.user,
        &connector_id,
        &mut auth_req,
        app.config.expiration,
    )
    .await
    .map_err(|err| redirect(&auth_request, err))?;
    Ok(Redirect::to(&redirect_uri))
}

fn redirect<E: ToString>(auth_request: &auth::AuthRequest, err: E) -> Redirect {
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
    Redirect::to(&redirect_uri)
}

#[derive(Template, Default)]
#[template(path = "password.html")]
pub struct Password {
    pub post_url: String,
    pub username: String,
    pub username_prompt: String,
    pub invalid: String,
    pub back_link: String,
}

async fn password_login_html(
    app: AppState,
    Path(connector_id): Path<String>,
    Valid(mut auth_req_id): Valid<password::AuthReqID>,
) -> Result<HtmlTemplate<Password>> {
    let auth_request =
        get_auth_request(&app.store.auth_request, &auth_req_id).await?;
    if !auth_request.connector_id.eq(&connector_id) {
        return Err(errors::bad_request("Requested resource does not exist."));
    }
    let connector = get_connector(&app.store.connector, &connector_id).await?;
    match open_connector(&app.store.user, &connector)? {
        oidc::Connector::Password(conn) => {
            auth_req_id.prompt = Some(conn.prompt().to_owned());
            Ok(HtmlTemplate(Password {
                post_url: format!(
                    "/auth/{}/login?{}",
                    connector_id,
                    serde_urlencoded::to_string(auth_req_id)
                        .map_err(errors::any)?
                ),
                username_prompt: format!("Enter your {}", conn.prompt()),
                ..Default::default()
            }))
        }
        _ => Err(errors::bad_request("unsupported connector type")),
    }
}

fn relogin_html<E: ToString>(
    connector_id: &str,
    auth_req_id: &password::AuthReqID,
    name: &str,
    err: E,
) -> HtmlTemplate<Password> {
    HtmlTemplate(Password {
        post_url: format!(
            "/auth/{}/login?{}",
            connector_id,
            serde_urlencoded::to_string(auth_req_id)
                .map_err(errors::any)
                .unwrap()
        ),
        username: name.to_string(),
        username_prompt: format!(
            "Enter your {}",
            auth_req_id.prompt.as_deref().unwrap_or("Username")
        ),
        invalid: err.to_string(),
        ..Default::default()
    })
}

async fn password_login_handle(
    app: AppState,
    Path(connector_id): Path<String>,
    Valid(auth_req_code): Valid<password::AuthReqID>,
    Valid(Form(login_data)): Valid<Form<LoginData>>,
) -> core::result::Result<Response, HtmlTemplate<Password>> {
    let mut auth_request =
        get_auth_request(&app.store.auth_request, &auth_req_code)
            .await
            .map_err(|err| {
                relogin_html(
                    &connector_id,
                    &auth_req_code,
                    &login_data.login,
                    err,
                )
            })?;
    if !auth_request.connector_id.eq(&connector_id) {
        return Err(relogin_html(
            &connector_id,
            &auth_req_code,
            &login_data.login,
            "Requested resource does not exist.",
        ));
    };
    let connector = get_connector(&app.store.connector, &connector_id)
        .await
        .map_err(|err| {
            relogin_html(&connector_id, &auth_req_code, &login_data.login, err)
        })?;
    let conn_impl =
        open_connector(&app.store.user, &connector).map_err(|err| {
            relogin_html(&connector_id, &auth_req_code, &login_data.login, err)
        })?;
    match conn_impl {
        oidc::Connector::Password(conn) => {
            let scopes = connect::parse_scopes(&auth_request.scopes);
            let identity = conn
                .login(
                    &scopes,
                    &connect::Info {
                        subject: login_data.login.clone(),
                        password: login_data.password,
                    },
                )
                .await
                .map_err(|err| {
                    relogin_html(
                        &connector_id,
                        &auth_req_code,
                        &login_data.login,
                        err,
                    )
                })?;

            let (mut redirect_uri, can_skip_approval) = finalize_login(
                &app.store.auth_request,
                &app.store.offline_session,
                &mut auth_request,
                &identity,
                conn.refresh_enabled(),
            )
            .await
            .map_err(|err| {
                relogin_html(
                    &connector_id,
                    &auth_req_code,
                    &login_data.login,
                    err,
                )
            })?;

            if can_skip_approval {
                auth_request =
                    get_auth_request(&app.store.auth_request, &auth_req_code)
                        .await
                        .map_err(|err| {
                            relogin_html(
                                &connector_id,
                                &auth_req_code,
                                &login_data.login,
                                err,
                            )
                        })?;

                redirect_uri = send_code(
                    &app.store.auth_request,
                    &app.access_token,
                    &app.store.auth_code,
                    &auth_request,
                )
                .await
                .map_err(|err| {
                    relogin_html(
                        &connector_id,
                        &auth_req_code,
                        &login_data.login,
                        err,
                    )
                })?;
            }
            let mut headers = HeaderMap::new();
            headers.insert(header::LOCATION, redirect_uri.parse().unwrap());
            Ok((StatusCode::SEE_OTHER, headers).into_response())
        }
        _ => Err(relogin_html(
            &connector_id,
            &auth_req_code,
            &login_data.login,
            "unsupported connector type",
        )),
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
    Valid(req_hmac): Valid<auth::ReqHmac>,
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
    Valid(Form(req_hmac)): Valid<Form<auth::ReqHmac>>,
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
