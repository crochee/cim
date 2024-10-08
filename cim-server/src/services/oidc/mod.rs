pub mod auth;
pub mod connect;
pub mod key;
pub mod password;
pub mod token;

use chrono::Utc;
use http::Uri;
use jsonwebkey as jwk;
use rand::Rng;

use cim_slo::{errors, Result};
use cim_storage::{
    authrequest, client, connector, user, Interface, WatchInterface,
};

use auth::AuthRequest;

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

pub async fn get_connector<C: Interface<T = connector::Connector>>(
    connector_store: &C,
    id: &str,
) -> Result<connector::Connector> {
    let mut connector = connector::Connector {
        id: id.to_string(),
        ..Default::default()
    };
    connector_store.get(&mut connector).await?;
    Ok(connector)
}

pub enum Connector {
    Password(Box<dyn connect::PasswordConnector + Send>),
    Callback(Box<dyn connect::CallbackConnector + Send>),
    Saml(Box<dyn connect::SAMLConnector + Send>),
}

pub fn open_connector<
    U: WatchInterface<T = user::User> + Send + Sync + Clone + 'static,
>(
    user_store: &U,
    conn: &connector::Connector,
) -> Result<Connector> {
    match conn.connector_type.as_str() {
        "cim" => {
            // let us = user_store.to_owned();
            Ok(Connector::Password(Box::new(connect::UserPassword::new(
                user_store.clone(),
            ))))
        }
        "mockCallback" => Ok(Connector::Callback(Box::new(
            connect::MockCallbackConnector::new(),
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

pub async fn run_connector<
    S: Interface<T = authrequest::AuthRequest>,
    U: WatchInterface<T = user::User> + Send + Sync + Clone + 'static,
>(
    auth_request_store: &S,
    conn: &connector::Connector,
    user_store: &U,
    connector_id: &str,
    auth_req: &mut authrequest::AuthRequest,
    expires_in: i64,
) -> Result<String> {
    let connector_impl = open_connector(user_store, conn)?;

    auth_req.id = uuid::Uuid::new_v4().to_string();
    auth_req.connector_id = connector_id.to_string();
    auth_req.expiry = Utc::now().timestamp() + expires_in;
    auth_request_store.put(auth_req, 0).await?;

    match connector_impl {
        Connector::Password(_) => Ok(format!(
            "/auth/{}/login?state={}",
            connector_id, auth_req.id
        )),
        Connector::Callback(cc) => {
            let scopes = connect::parse_scopes(&auth_req.scopes);
            tracing::debug!("{:?}", scopes);
            cc.login_url(&scopes, "/callback", &auth_req.id).await
        }
        Connector::Saml(_) => Ok("".to_string()),
    }
}

pub async fn valid_scope<C: Interface<T = client::Client>>(
    client_store: &C,
    client_id: &str,
    scopes: &Vec<String>,
) -> Result<Vec<String>> {
    let mut has_open_id_scope = false;
    let mut unrecognized = Vec::new();
    let mut invalid_scopes = Vec::new();
    let mut audience = Vec::new();
    for scope in scopes {
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
                if !client_id.eq(peer_id) {
                    invalid_scopes.push(scope.clone());
                    continue;
                }
                let mut client_value = client::Client {
                    id: peer_id.to_string(),
                    ..Default::default()
                };
                if let Err(err) = client_store.get(&mut client_value).await {
                    if err.eq(&errors::not_found("")) {
                        invalid_scopes.push(scope.clone());
                    }
                    return Err(err);
                }
                let mut trusted_peers = false;
                for id in client_value.trusted_peers {
                    if id.eq(&client_id) {
                        trusted_peers = true;
                        break;
                    }
                }
                if !trusted_peers {
                    invalid_scopes.push(scope.clone());
                    continue;
                }
                audience.push(peer_id.to_owned());
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
    if audience.is_empty() {
        audience.push(client_id.to_owned());
    }

    Ok(audience)
}

pub async fn parse_auth_request<C: Interface<T = client::Client>>(
    client_store: &C,
    req: &AuthRequest,
) -> Result<authrequest::AuthRequest> {
    let scopes: Vec<String> =
        req.scope.split_whitespace().map(|x| x.to_owned()).collect();
    valid_scope(client_store, &req.client_id, &scopes).await?;

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
    let mut client = client::Client {
        id: req.client_id.clone(),
        ..Default::default()
    };
    client_store.get(&mut client).await?;
    if !validate_redirect_uri(&client, &req.redirect_uri) {
        return Err(errors::bad_request("Invalid redirect_uri"));
    }
    let hmac_key = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect::<String>();

    Ok(authrequest::AuthRequest {
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
    if !client_value.account_id.is_empty()
        || !client_value.redirect_uris.is_empty()
    {
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

pub fn jwk_to_public(key: Box<jwk::Key>) -> Result<Box<jwk::Key>> {
    if !key.is_private() {
        return Ok(key);
    }
    Ok(Box::new(match *key {
        jwk::Key::Symmetric { .. } => {
            return Err(errors::bad_request("not supported symmetric key"))
        }
        jwk::Key::EC {
            curve: jwk::Curve::P256 { x, y, .. },
        } => jwk::Key::EC {
            curve: jwk::Curve::P256 {
                x: x.clone(),
                y: y.clone(),
                d: None,
            },
        },
        jwk::Key::RSA { public, .. } => jwk::Key::RSA {
            public: public.clone(),
            private: None,
        },
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    use async_trait::async_trait;
    use cim_storage::{client::Client, Interface, List};
    use mockall::mock;

    mock! {
        pub ClientStore {
            fn get(&self, output: &mut Client) -> Result<()>;
        }
    }

    #[async_trait]
    impl Interface for MockClientStore {
        type T = Client;
        type L = ();
        async fn put(&self, _input: &Self::T, _ttl: u64) -> Result<()> {
            unimplemented!()
        }
        async fn delete(&self, _input: &Self::T) -> Result<()> {
            unimplemented!()
        }
        async fn get(&self, output: &mut Self::T) -> Result<()> {
            self.get(output)
        }
        async fn list(
            &self,
            _opts: &Self::L,
            _output: &mut List<Self::T>,
        ) -> Result<()> {
            unimplemented!()
        }
        async fn count(&self, _opts: &Self::L, _unscoped: bool) -> Result<i64> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn test_parse_auth_request() {
        let mut client_store = MockClientStore::new();
        client_store.expect_get().returning(|output| {
            output.id = "client_id".to_owned();
            Ok(())
        });

        let req = AuthRequest {
            client_id: "client_id".to_owned(),
            redirect_uri: "http://localhost:3000/callback".to_owned(),
            response_type: "code id_token token".to_owned(),
            scope: "openid".to_owned(),
            nonce: Some("nonce".to_owned()),
            state: "state".to_owned(),
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
