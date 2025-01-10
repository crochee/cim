use base64::engine::{general_purpose, Engine};
use chrono::Utc;
use http::Uri;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use validator::Validate;

use cim_slo::{errors, next_id, Result};
use cim_storage::{authcode, authrequest, offlinesession, Interface, List};

use super::{connect::Identity, token};

#[derive(Debug, Validate, Deserialize)]
pub struct LoginData {
    pub login: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct AuthReqID {
    #[validate(length(min = 1))]
    pub state: String,
    #[validate(length(min = 1))]
    pub prompt: Option<String>,
}

pub async fn get_auth_request<S: Interface<T = authrequest::AuthRequest>>(
    auth_request_store: &S,
    req: &AuthReqID,
) -> Result<authrequest::AuthRequest> {
    let mut auth_request = authrequest::AuthRequest {
        id: req.state.clone(),
        ..Default::default()
    };
    auth_request_store.get(&mut auth_request).await?;
    Ok(auth_request)
}

pub async fn finalize_login<
    S: Interface<T = authrequest::AuthRequest>,
    O: Interface<
        T = offlinesession::OfflineSession,
        L = offlinesession::ListParams,
    >,
>(
    auth_request_store: &S,
    offline_session_store: &O,
    auth_req: &mut authrequest::AuthRequest,
    identity: &Identity,
    is_refresh: bool,
) -> Result<(String, bool)> {
    auth_req.logged_in = true;
    auth_req.claim = identity.claim.clone();
    auth_req.connector_data = identity.connector_data.clone();

    auth_request_store.put(auth_req).await?;
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
    if !is_refresh {
        return Ok((return_url, false));
    }
    if auth_req.scopes.contains(&String::from("offline_access")) {
        return Ok((return_url, false));
    }
    let mut sessions = List::default();
    offline_session_store
        .list(
            &offlinesession::ListParams {
                user_id: Some(auth_req.claim.sub.clone()),
                conn_id: Some(auth_req.connector_id.clone()),
                pagination: cim_storage::Pagination {
                    count_disable: true,
                    ..Default::default()
                },
            },
            &mut sessions,
        )
        .await?;
    if sessions.data.is_empty() {
        let id = next_id().map_err(errors::any)?;
        offline_session_store
            .put(&offlinesession::OfflineSession {
                id: id.to_string(),
                user_id: auth_req.claim.sub.clone(),
                conn_id: auth_req.connector_id.clone(),
                connector_data: auth_req.connector_data.clone(),
                ..Default::default()
            })
            .await?;
    } else {
        let mut session = sessions.data.remove(0);
        if let Some(connector_data) = &auth_req.connector_data {
            session.connector_data = Some(connector_data.clone());
            offline_session_store.put(&session).await?;
        }
    }
    Ok((return_url, false))
}

pub async fn send_code<
    S: Interface<T = authrequest::AuthRequest>,
    T: token::Token,
    C: Interface<T = authcode::AuthCode>,
>(
    auth_request_store: &S,
    token_creater: &T,
    authcode_store: &C,
    auth_request: &authrequest::AuthRequest,
) -> Result<String> {
    if Utc::now().timestamp() > auth_request.expiry {
        return Err(errors::bad_request("User session has expired."));
    }
    let mut auth_request = authrequest::AuthRequest::default();
    auth_request.id = auth_request.id.clone();

    auth_request_store.delete(&auth_request).await?;
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
                let code_val = authcode::AuthCode {
                    id: next_id().map_err(errors::any)?.to_string(),
                    ..Default::default()
                };
                code = Some(code_val.clone());
                authcode_store.put(&code_val).await?;
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
