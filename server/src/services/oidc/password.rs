use http::Uri;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use super::auth::AuthRequest;
use slo::{errors, Result};

#[derive(Debug, Validate, Deserialize)]
pub struct LoginData {
    pub login: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct AuthCode {
    pub code: String,
    pub state: String,
}

pub async fn password_login(
    req: &AuthRequest,
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

    let mut redirect_uri = req.redirect_uri.clone();
    let uri = redirect_uri.parse::<Uri>().map_err(errors::any)?;
    if uri.query().is_none() {
        redirect_uri.push('?');
    } else {
        redirect_uri.push('&');
    }
    redirect_uri.push_str(
        &serde_urlencoded::to_string(AuthCode {
            code: "xxxcode".to_string(),
            state: req.state.clone(),
        })
        .map_err(errors::any)?,
    );
    Ok(redirect_uri)
}
