use http::Uri;
use rand::Rng;
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
        &serde_urlencoded::to_string(AuthCode {
            code: "xxxcode".to_string(),
            state: req.state.clone(),
        })
        .map_err(errors::any)?,
    );
    Ok(redirect_uri)
}

const CODE_CHALLENGE_METHOD_PLAIN: &str = "plain";
const CODE_CHALLENGE_METHOD_S256: &str = "S256";
const SCOPE_OFFLINE_ACCESS: &str = "offline_access"; // Request a refresh token.
const SCOPE_OPENID: &str = "openid";
const SCOPE_GROUPS: &str = "groups";
const SCOPE_EMAIL: &str = "email";
const SCOPE_PROFILE: &str = "profile";
const SCOPE_FEDERATED_ID: &str = "federated:id";
const SCOPE_CROSS_CLIENT_PREFIX: &str = "audience:server:client_id:";

async fn parse_auth_request<C: client::ClientStore>(
    client_store: &C,
    req: &mut AuthRequest,
) -> Result<authrequest::AuthRequest> {
    let scopes: Vec<String> =
        req.scope.split_whitespace().map(|x| x.to_owned()).collect();

    let mut has_open_id_scope = false;
    let mut unrecognized = Vec::new();
    let mut invalid_scopes = Vec::new();
    for scope in &scopes {
        match scope.as_str() {
            "openid" => has_open_id_scope = true,
            SCOPE_OFFLINE_ACCESS
            | SCOPE_OPENID
            | SCOPE_EMAIL
            | SCOPE_PROFILE
            | SCOPE_GROUPS
            | SCOPE_FEDERATED_ID
            | SCOPE_CROSS_CLIENT_PREFIX => {}
            _ => {
                if !scope.starts_with(SCOPE_CROSS_CLIENT_PREFIX) {
                    unrecognized.push(scope.clone());
                    continue;
                }
                let peer_id =
                    scope.trim_start_matches(SCOPE_CROSS_CLIENT_PREFIX);
                if !req.client_id.eq(peer_id) {
                    invalid_scopes.push(scope.clone());
                    continue;
                }
                match client_store.get_client(peer_id, None).await {
                    Ok(client_value) => {
                        let mut trusted_peers = false;
                        for id in client_value.trusted_peers {
                            if id.eq(&req.client_id) {
                                trusted_peers = true;
                                break;
                            }
                        }
                        if !trusted_peers {
                            invalid_scopes.push(scope.clone());
                        }
                    }
                    Err(err) => {
                        if err.eq(&errors::not_found("")) {
                            invalid_scopes.push(scope.clone());
                        }
                        return Err(err);
                    }
                }
            }
        }
    }
    if !has_open_id_scope {
        return Err(errors::bad_request(
            r#"Missing required scope(s) ["openid"]."#,
        ));
    }

    let response_types: Vec<String> = req
        .response_type
        .split_whitespace()
        .map(|x| x.to_owned())
        .collect();

    let mut has_code = false;
    let mut has_id_token = false;
    let mut has_token = false;
    for response_type in &response_types {
        if response_type.eq("code") {
            has_code = true;
        } else if response_type.eq("id_token") {
            has_id_token = true;
        } else if response_type.eq("token") {
            has_token = true;
        }
    }
    if !has_code && !has_id_token && !has_token {
        return Err(errors::bad_request(
            r#"Missing required response_type(s) ["code", "token", "id_token"]."#,
        ));
    }

    let mut code_challenge_method = String::from(CODE_CHALLENGE_METHOD_PLAIN);
    if let Some(value) = &req.code_challenge_method {
        match value.as_str() {
            CODE_CHALLENGE_METHOD_PLAIN | CODE_CHALLENGE_METHOD_S256 => {
                code_challenge_method = value.to_string()
            }
            _ => {
                return Err(errors::bad_request(
                    "Invalid code_challenge_method value",
                ));
            }
        }
    }

    let hmac_key = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect::<String>();

    let mut auth_req = authrequest::AuthRequest {
        id: uuid::Uuid::new_v4().to_string(),
        client_id: req.client_id.clone(),
        response_types,
        scopes,
        redirect_uri: req.redirect_uri.clone(),
        code_challenge: req.code_challenge.clone(),
        code_challenge_method,
        nonce: req.nonce.clone(),
        state: req.state.clone(),
        hmac_key,
        force_approval_prompt: req.skip_approval.unwrap_or_default(),
        ..Default::default()
    };
    Ok(auth_req)
}

#[cfg(test)]
mod tests {
    use super::*;

    use storage::client::*;

    #[tokio::test]
    async fn test_parse_auth_request() {
        let client_store = MockClientStore::new();

        let mut req = AuthRequest {
            client_id: "client_id".to_owned(),
            redirect_uri: "http://localhost:3000/callback".to_owned(),
            response_type: "code id_token token".to_owned(),
            scope: "openid".to_owned(),
            nonce: "nonce".to_owned(),
            state: "state".to_owned(),
            back: None,
            code_challenge: "code_challenge".to_owned(),
            code_challenge_method: None,
            skip_approval: None,
            connector_id: None,
        };
        let auth_req =
            parse_auth_request(&client_store, &mut req).await.unwrap();
        assert_eq!(auth_req.scopes.len(), 1);
        assert_eq!(auth_req.scopes[0], "openid");
    }
}
