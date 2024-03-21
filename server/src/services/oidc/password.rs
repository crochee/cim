use http::Uri;
use serde::{Deserialize, Serialize};
use storage::{authrequest, client};
use utoipa::ToSchema;
use validator::Validate;

use super::auth::AuthRequest;
use slo::{errors, Result};

#[derive(Debug, Validate, Deserialize)]
pub struct LoginData {
    pub login: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
pub struct AuthReqID {
    #[validate(length(min = 1))]
    pub state: String,
    #[validate(length(min = 1))]
    pub prompt: Option<String>,
}

pub async fn get_auth_request<S: authrequest::AuthRequestStore>(
    auth_request_store: &S,
    req: &AuthReqID,
) -> Result<authrequest::AuthRequest> {
    auth_request_store.get_auth_request(&req.state).await
}

pub async fn run_password_connector<S: authrequest::AuthRequestStore>(
    _auth_request_store: &S,
    _req: &AuthReqID,
) -> Result<String> {
    Ok("".to_string())
}

pub async fn password_login<C: client::ClientStore>(
    _client_store: &C,
    req: &mut AuthRequest,
    _login_data: &LoginData,
) -> Result<String> {
    let mut implicit_or_hybrid = false;
    match req.response_type.as_str() {
        "code" => {}
        "token" => implicit_or_hybrid = true,
        _ => return Err(errors::bad_request("invalid response_type")),
    }
    if !implicit_or_hybrid {
        return Err(errors::bad_request("invalid response_type"));
    }
    // 跳转到授权界面
    if !req.skip_approval.unwrap_or_default() {
        return Ok("".to_string());
    }

    let mut redirect_uri = req.redirect_uri.clone();
    let uri = redirect_uri.parse::<Uri>().map_err(errors::any)?;
    if uri.query().is_none() {
        redirect_uri.push('?');
    } else {
        redirect_uri.push('&');
    }
    redirect_uri.push_str(
        &serde_urlencoded::to_string(AuthReqID {
            state: req.state.clone(),
            prompt: None,
        })
        .map_err(errors::any)?,
    );
    Ok(redirect_uri)
}
