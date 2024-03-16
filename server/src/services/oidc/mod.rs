pub mod auth;
pub mod key;
pub mod password;

use http::Uri;
use rand::Rng;
use slo::{errors, Result};
use storage::{authrequest, client, connector};

use auth::AuthRequest;

use super::authentication::connect;

const CODE_CHALLENGE_METHOD_PLAIN: &str = "plain";
const CODE_CHALLENGE_METHOD_S256: &str = "S256";

const SCOPE_OFFLINE_ACCESS: &str = "offline_access"; // Request a refresh token.
const SCOPE_OPENID: &str = "openid";
const SCOPE_GROUPS: &str = "groups";
const SCOPE_EMAIL: &str = "email";
const SCOPE_PROFILE: &str = "profile";
const SCOPE_FEDERATED_ID: &str = "federated:id";
const SCOPE_CROSS_CLIENT_PREFIX: &str = "audience:server:client_id:";

const RESPONSE_TYPE_CODE: &str = "code"; // "Regular" flow
const RESPONSE_TYPE_TOKEN: &str = "token"; // Implicit flow for frontend apps.
const RESPONSE_TYPE_IDTOKEN: &str = "id_token"; // ID Token in url fragment

pub async fn get_connector<C: connector::ConnectorStore>(
    connector_store: &C,
    id: &str,
) -> Result<connector::Connector> {
    connector_store.get_connector(id).await
}

pub enum Connector {
    Password(Box<dyn connect::PasswordConnector>),
    Callback(Box<dyn connect::CallbackConnector>),
    Saml(Box<dyn connect::SAMLConnector>),
}

pub fn open_connector(conn: &connector::Connector) -> Result<Connector> {
    match conn.connector_type.as_str() {
        "cim" => Ok(Connector::Password(Box::new(
            connect::MockPasswordConnector::new(),
        ))),
        "mockCallback" => Ok(Connector::Password(Box::new(
            connect::MockPasswordConnector::new(),
        ))),
        "mockPassword" => Ok(Connector::Password(Box::new(
            connect::MockPasswordConnector::new(),
        ))),
        "saml" => {
            Ok(Connector::Saml(Box::new(connect::MockSAMLConnector::new())))
        }
        _ => Err(errors::bad_request("unsupported connector type")),
    }
}

pub async fn parse_auth_request<C: client::ClientStore>(
    client_store: &C,
    req: &AuthRequest,
) -> Result<authrequest::AuthRequest> {
    let scopes: Vec<String> =
        req.scope.split_whitespace().map(|x| x.to_owned()).collect();

    let mut has_open_id_scope = false;
    let mut unrecognized = Vec::new();
    let mut invalid_scopes = Vec::new();
    for scope in &scopes {
        match scope.as_str() {
            SCOPE_OPENID => has_open_id_scope = true,
            SCOPE_OFFLINE_ACCESS
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
    if !unrecognized.is_empty() {
        return Err(errors::bad_request(&format!(
            r#"Unrecognized scope(s) {:?}"#,
            unrecognized
        )));
    }
    if !invalid_scopes.is_empty() {
        return Err(errors::bad_request(&format!(
            r#"Client can't request scope(s) {:?}"#,
            invalid_scopes
        )));
    }
    let response_types: Vec<String> = req
        .response_type
        .split_whitespace()
        .map(|x| x.to_owned())
        .collect();
    if response_types.is_empty() {
        return Err(errors::bad_request("no response_type provided"));
    }

    let mut has_code = false;
    let mut has_id_token = false;
    let mut has_token = false;
    for response_type in &response_types {
        match response_type.as_str() {
            RESPONSE_TYPE_CODE => has_code = true,
            RESPONSE_TYPE_TOKEN => has_token = true,
            RESPONSE_TYPE_IDTOKEN => has_id_token = true,
            _ => {
                return Err(errors::bad_request(&format!(
                    r#"Invalid response_type value {response_type}"#
                )));
            }
        }
    }
    if !has_code && !has_id_token && has_token {
        return Err(errors::bad_request(
            r#"response_type 'token' must be provided with ["code", "id_token"]."#,
        ));
    }
    let nonce = req.nonce.clone().unwrap_or_default();
    if !has_code && nonce.is_empty() {
        return Err(errors::bad_request(
            r#"response_type 'token' requires a nonce."#,
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
    let client = client_store.get_client(&req.client_id, None).await?;
    if !validate_redirect_uri(&client, &req.redirect_uri) {
        return Err(errors::bad_request("Invalid redirect_uri"));
    }
    let hmac_key = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect::<String>();

    Ok(authrequest::AuthRequest {
        id: uuid::Uuid::new_v4().to_string(),
        client_id: req.client_id.clone(),
        response_types,
        scopes,
        redirect_uri: req.redirect_uri.clone(),
        code_challenge: req.code_challenge.clone(),
        code_challenge_method,
        nonce,
        state: req.state.clone(),
        hmac_key,
        force_approval_prompt: req.skip_approval.unwrap_or_default(),
        ..Default::default()
    })
}

fn validate_redirect_uri(
    client_value: &client::Client,
    redirect_uri: &String,
) -> bool {
    for uri in client_value.redirect_uris.iter() {
        if redirect_uri.eq(uri) {
            return true;
        }
    }
    if !client_value.public || !client_value.redirect_uris.is_empty() {
        return false;
    }
    let u = match redirect_uri.parse::<Uri>() {
        Ok(v) => v,
        Err(_) => return false,
    };
    if Some("http") != u.scheme_str() {
        return false;
    }
    if Some("localhost") == u.host() {
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    use storage::client::*;

    #[tokio::test]
    async fn test_parse_auth_request() {
        let mut client_store = MockClientStore::new();
        client_store.expect_get_client().returning(|_, _| {
            Ok(Client {
                id: "client_id".to_owned(),
                public: true,
                ..Default::default()
            })
        });

        let mut req = AuthRequest {
            client_id: "client_id".to_owned(),
            redirect_uri: "http://localhost:3000/callback".to_owned(),
            response_type: "code id_token token".to_owned(),
            scope: "openid".to_owned(),
            nonce: Some("nonce".to_owned()),
            state: "state".to_owned(),
            back: None,
            code_challenge: "R".to_owned(),
            code_challenge_method: None,
            skip_approval: None,
            connector_id: None,
        };
        let auth_req = parse_auth_request(&client_store, &req).await.unwrap();
        assert_eq!(auth_req.response_types, vec!["code", "id_token", "token"]);
        assert_eq!(auth_req.scopes, vec!["openid"]);
        assert_eq!(auth_req.redirect_uri, "http://localhost:3000/callback");
        assert_eq!(auth_req.nonce, "nonce");
        assert_eq!(auth_req.state, "state");
        assert!(!auth_req.force_approval_prompt);
        assert_eq!(auth_req.code_challenge, "R");
        assert_eq!(auth_req.code_challenge_method, "plain");
    }
}
