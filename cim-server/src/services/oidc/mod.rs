pub mod auth;
pub mod connect;
pub mod key;
pub mod token;

use axum::extract::Request;
use chrono::Utc;
use http::Uri;
use jsonwebkey as jwk;

use cim_slo::{errors, next_id, Result};
use cim_storage::{
    authcode, authrequest, client, connector, offlinesession, user, Interface,
    List, WatchInterface,
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

pub async fn get_auth_request<C: Interface<T = authrequest::AuthRequest>>(
    auth_request_store: &C,
    id: &str,
) -> Result<authrequest::AuthRequest> {
    let mut auth_req = authrequest::AuthRequest {
        id: id.to_string(),
        ..Default::default()
    };
    auth_request_store.get(&mut auth_req).await?;
    Ok(auth_req)
}

pub fn open_connector<
    U: WatchInterface<T = user::User> + Send + Sync + Clone + 'static,
>(
    user_store: &U,
    conn: Option<&connector::Connector>,
) -> Result<Box<dyn connect::CallbackConnector + Send>> {
    match conn {
        Some(conn) => match conn.connector_type.as_str() {
            "cim" | "local" => {
                Ok(Box::new(connect::UserPassword::new(user_store.clone())))
            }
            "mockCallback" => {
                Ok(Box::new(connect::MockCallbackConnector::new()))
            }
            _ => Err(errors::bad_request("unsupported connector type")),
        },
        None => Ok(Box::new(connect::UserPassword::new(user_store.clone()))),
    }
}

pub async fn redirect_auth_page<
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
    let connector_impl = open_connector(user_store, Some(conn))?;
    auth_req.id = uuid::Uuid::new_v4().to_string();
    auth_req.connector_id = connector_id.to_string();
    auth_req.connector_data = conn.connector_data.clone();
    auth_req.expiry = Utc::now().timestamp() + expires_in;
    auth_request_store.put(auth_req).await?;
    connector_impl
        .login_url(&auth_req.scopes, "/callback", &auth_req.id)
        .await
}

pub async fn auth_page_callback<
    S: Interface<T = authrequest::AuthRequest>,
    A: Interface<T = authcode::AuthCode>,
    C: Interface<T = connector::Connector>,
    O: Interface<
        T = offlinesession::OfflineSession,
        L = offlinesession::ListParams,
    >,
    T: token::Token,
    U: WatchInterface<T = user::User> + Send + Sync + Clone + 'static,
>(
    auth_request_store: &S,
    user_store: &U,
    auth_code_store: &A,
    connector_store: &C,
    offline_session_store: &O,
    token_creater: &T,
    auth_request_state: &auth::AuthRequestState,
    req: Request,
) -> Result<String> {
    let mut auth_req = authrequest::AuthRequest {
        id: auth_request_state.state.clone(),
        ..Default::default()
    };
    auth_request_store.get(&mut auth_req).await?;

    let mut connector = connector::Connector {
        id: auth_req.connector_id.clone(),
        ..Default::default()
    };
    connector_store.get(&mut connector).await?;
    let connector_impl = open_connector(user_store, Some(&connector))?;

    let identity = connector_impl
        .handle_callback(&auth_req.scopes, req)
        .await?;

    auth_req.claim = identity.claim.clone();
    auth_req.connector_data = identity.connector_data.clone();

    auth_request_store.put(&auth_req).await?;

    if auth_req.scopes.contains(&String::from("offline_access"))
        && connector_impl.support_refresh()
    {
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
    }
    send_code(
        auth_request_store,
        token_creater,
        auth_code_store,
        &auth_req,
    )
    .await
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
    auth_request_store.delete(auth_request).await?;
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
            RESPONSE_TYPE_CODE => {
                let auth_code = authcode::AuthCode {
                    id: uuid::Uuid::new_v4().to_string(),
                    client_id: auth_request.client_id.clone(),
                    scopes: auth_request.scopes.clone(),
                    nonce: auth_request.nonce.clone(),
                    redirect_uri: auth_request.redirect_uri.clone(),
                    code_challenge: auth_request.code_challenge.clone(),
                    code_challenge_method: auth_request
                        .code_challenge_method
                        .clone(),
                    claim: auth_request.claim.clone(),
                    connector_id: auth_request.connector_id.clone(),
                    connector_data: auth_request.connector_data.clone(),
                    expiry: Utc::now().timestamp() + 30 * 60,
                    ..Default::default()
                };
                code = Some(auth_code.id.clone());
                authcode_store.put(&auth_code).await?;
            }
            RESPONSE_TYPE_IDTOKEN => {
                implicit_or_hybrid = true;
                // TODO:claims and token opts fill
                let mut claims = token::Claims {
                    claim: auth_request.claim.clone(),
                    nonce: auth_request.nonce.clone(),
                    aud: auth_request.client_id.clone(),
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
            RESPONSE_TYPE_TOKEN => {
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
        if let Some(id) = code {
            query.push_str("&code=");
            query.push_str(&id);
        }
    } else if let Some(id) = code {
        query.push_str("code=");
        query.push_str(&id);
        query.push_str("&state=");
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
    if !has_code && req.nonce.is_empty() {
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

    Ok(authrequest::AuthRequest {
        client_id: req.client_id.clone(),
        response_types,
        scopes,
        redirect_uri: req.redirect_uri.clone(),
        code_challenge: req.code_challenge.clone(),
        code_challenge_method,
        nonce: req.nonce.clone(),
        state: req.state.clone(),
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
        async fn put(&self, _input: &Self::T) -> Result<()> {
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
            nonce: "nonce".to_owned(),
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
        assert_eq!(auth_req.code_challenge, "R");
        assert_eq!(auth_req.code_challenge_method, "plain");
    }
}
