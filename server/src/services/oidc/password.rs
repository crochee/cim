use base64::engine::{general_purpose, Engine};
use chrono::Utc;
use http::Uri;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use storage::{authrequest, client};
use utoipa::ToSchema;
use validator::Validate;

use super::{auth::AuthRequest, connect::Identity};
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
pub async fn finalize_login<S: authrequest::AuthRequestStore>(
    auth_request_store: &S,
    auth_req: &authrequest::AuthRequest,
    identity: &Identity,
    is_refresh: bool,
) -> Result<(String, bool)> {
    auth_request_store
        .put_auth_request(
            Some(auth_req.id.clone()),
            &authrequest::Content {
                client_id: auth_req.client_id.clone(),
                response_types: auth_req.response_types.clone(),
                scopes: auth_req.scopes.clone(),
                redirect_uri: auth_req.redirect_uri.clone(),
                code_challenge: auth_req.code_challenge.clone(),
                code_challenge_method: auth_req.code_challenge_method.clone(),
                nonce: auth_req.nonce.clone(),
                state: auth_req.state.clone(),
                hmac_key: auth_req.hmac_key.clone(),
                force_approval_prompt: auth_req.force_approval_prompt,
                logged_in: auth_req.logged_in,
                claims_user_id: identity.user_id.clone(),
                claims_user_name: identity.username.clone(),
                claims_email: identity.email.clone().unwrap_or_default(),
                claims_email_verified: identity.email_verified,
                claims_groups: auth_req.claims_groups.clone(),
                claims_preferred_username: identity.preferred_username.clone(),
                connector_id: auth_req.connector_id.clone(),
                connector_data: Some(identity.connector_data.to_string()),
                expires_in: auth_req.expires_in,
            },
        )
        .await?;
    if !auth_req.force_approval_prompt {
        return Ok(("".to_string(), true));
    }

    let hmac = Sha256::new_with_prefix(&auth_req.hmac_key)
        .chain_update(&auth_req.id)
        .finalize();
    let mut return_url = String::from("/approval?req=");
    return_url.push_str(&auth_req.id);
    return_url.push_str("&hmac=");
    let hmac_str = general_purpose::URL_SAFE_NO_PAD.encode(hmac);
    return_url.push_str(&hmac_str);
    if is_refresh {
        // TODO: Add support for refresh tokens
    }
    Ok((return_url, false))
}

pub async fn send_code<S: authrequest::AuthRequestStore>(
    auth_request_store: &S,
    auth_request: &authrequest::AuthRequest,
) -> Result<String> {
    if Utc::now().timestamp() > auth_request.expires_in {
        return Err(errors::bad_request("User session has expired."));
    }
    auth_request_store
        .delete_auth_request(&auth_request.id)
        .await?;
    let u = auth_request.redirect_uri.parse::<Uri>().map_err(|err| {
        errors::bad_request(format!("Invalid redirect_uri. {}", err).as_str())
    })?;

    for response_type in &auth_request.response_types {}

    Ok("".to_string())
}
