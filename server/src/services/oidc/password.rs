use base64::engine::{general_purpose, Engine};
use chrono::Utc;
use http::Uri;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use storage::{authcode, authrequest};
use utoipa::ToSchema;
use validator::Validate;

use super::{connect::Identity, token};
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
    auth_req: &mut authrequest::AuthRequest,
    identity: &Identity,
    is_refresh: bool,
) -> Result<(String, bool)> {
    auth_req.logged_in = true;
    auth_req.claim = identity.claim.clone();
    auth_req.connector_data = Some(identity.connector_data.to_string());

    auth_request_store.put_auth_request(auth_req).await?;
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

pub async fn send_code<
    S: authrequest::AuthRequestStore,
    T: token::Token,
    C: authcode::AuthCodeStore,
>(
    auth_request_store: &S,
    token_creater: &T,
    authcode_store: &C,
    auth_request: &authrequest::AuthRequest,
) -> Result<String> {
    if Utc::now().timestamp() > auth_request.expiry {
        return Err(errors::bad_request("User session has expired."));
    }
    auth_request_store
        .delete_auth_request(&auth_request.id)
        .await?;
    let u = auth_request.redirect_uri.parse::<Uri>().map_err(|err| {
        errors::bad_request(format!("Invalid redirect_uri. {}", err).as_str())
    })?;
    let mut implicit_or_hybrid = false;
    let mut id_token = String::new();
    let mut id_token_expiry = 0;
    let mut code = None;
    let mut access_token = String::new();
    for response_type in &auth_request.response_types {
        match response_type.as_str() {
            super::RESPONSE_TYPE_CODE => {
                let code_val = authcode::AuthCode::default();
                code = Some(code_val.clone());
                authcode_store.put_auth_code(&code_val).await?;
            }
            super::RESPONSE_TYPE_IDTOKEN => {
                implicit_or_hybrid = true;
                // TODO:claims and token opts fill
                let mut claims = token::Claims {
                    claim: auth_request.claim.clone(),
                    ..Default::default()
                };

                let (access_token_val, _) =
                    token_creater.token(&claims).await?;
                claims.access_token = Some(access_token_val.clone());
                access_token = access_token_val;

                let (id_token_val, id_token_expiry_val) =
                    token_creater.token(&claims).await?;
                id_token = id_token_val;
                id_token_expiry = id_token_expiry_val;
            }
            super::RESPONSE_TYPE_TOKEN => {
                implicit_or_hybrid = true;
            }
            _ => {}
        }
    }
    let mut query = String::new();
    if implicit_or_hybrid {
        query.push_str("access_token=");
        query.push_str(&access_token);
        query.push_str("&token_type=bearer&state=");
        query.push_str(&auth_request.state);

        if !id_token.is_empty() {
            query.push_str("&id_token=");
            query.push_str(&id_token);
            if code.is_none() {
                query.push_str("&expires_in=");
                let expires_in = id_token_expiry - Utc::now().timestamp();
                query.push_str(&expires_in.to_string());
            }
        }
        if let Some(c) = code {
            query.push_str("&code=");
            query.push_str(&c.id);
        }
    } else if let Some(c) = code {
        query.push_str("code=");
        query.push_str(&c.id);
        query.push_str("state=");
        query.push_str(&auth_request.state);
    }
    let mut ru = auth_request.redirect_uri.clone();
    if u.query().is_some() {
        ru.push('&');
    } else {
        ru.push('?');
    }
    ru.push_str(query.as_str());
    Ok(ru)
}
