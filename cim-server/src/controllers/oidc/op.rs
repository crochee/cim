use axum::{
    extract::RawForm,
    routing::{get, post},
    Json, Router,
};
use axum_extra::{
    headers::{authorization::Basic, Authorization},
    TypedHeader,
};
use chrono::{Duration, Utc};
use http::{header, HeaderMap, StatusCode};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use cim_slo::{errors, Result};
use cim_storage::keys::KeyStore;

use crate::{
    services::oidc::{
        auth::{auth, AuthRequest},
        key, token,
    },
    valid::Valid,
    AppState,
};

pub fn new_router(state: AppState) -> Router {
    Router::new()
        // op common api
        .route("/.well-known/openid-configuration", get(discovery_handler))
        .route("/jwks", get(jwk_handler))
        // auth api
        .route("/auth", get(auth_handler))
        .route("/token", post(token_handler))
        .route("/userinfo", get(userinfo))
        .with_state(state)
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OpenIDConfiguration {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub jwks_uri: String,
    pub userinfo_endpoint: String,
    pub device_authorization_endpoint: String,
    pub id_token_signing_alg_values_supported: Vec<String>,
    pub grant_types_supported: Vec<String>,
    pub subject_types_supported: Vec<String>,
    pub response_types_supported: Vec<String>,
    pub scopes_supported: Vec<String>,
    pub token_endpoint_auth_methods_supported: Vec<String>,
    pub code_challenge_methods_supported: Vec<String>,
    pub claims_supported: Vec<String>,
}

async fn discovery_handler(app: AppState) -> Json<OpenIDConfiguration> {
    let issuer = format!("http://{}:{}", app.config.endpoint, app.config.port);
    Json(OpenIDConfiguration {
        issuer: issuer.to_string(),
        authorization_endpoint: format!("{issuer}/auth"),
        token_endpoint: format!("{issuer}/token"),
        jwks_uri: format!("{issuer}/jwks"),
        userinfo_endpoint: format!("{issuer}/userinfo"),
        device_authorization_endpoint: format!("{issuer}/device/code"),
        id_token_signing_alg_values_supported: vec!["RS256".to_string()],
        grant_types_supported: vec![
            "authorization_code".to_string(),
            "refresh_token".to_string(),
            "urn:ietf:params:oauth:grant-type:device_code".to_string(),
            "urn:ietf:params:oauth:grant-type:jwt-bearer".to_string(),
        ],
        subject_types_supported: vec!["public".to_string()],
        response_types_supported: vec![
            "code".to_string(),
            "token".to_string(),
            "id_token".to_string(),
        ],
        scopes_supported: vec![
            "openid".to_string(),
            "profile".to_string(),
            "email".to_string(),
        ],
        token_endpoint_auth_methods_supported: vec![
            "client_secret_basic".to_string()
        ],
        code_challenge_methods_supported: vec![
            "plain".to_string(),
            "S256".to_string(),
        ],
        claims_supported: vec![
            "aud".to_string(),
            "email".to_string(),
            "email_verified".to_string(),
            "exp".to_string(),
            "iat".to_string(),
            "iss".to_string(),
            "name".to_string(),
            "picture".to_string(),
            "sub".to_string(),
            "locale".to_string(),
            "at_hash".to_string(),
            "preferred_username".to_string(),
        ],
    })
}

async fn jwk_handler(
    app: AppState,
) -> Result<(http::HeaderMap, Json<key::JsonWebKeySet>)> {
    let keys = app.store.key.get_key().await?;
    let mut jwks = key::JsonWebKeySet {
        keys: Vec::with_capacity(keys.verification_keys.len() + 1),
    };
    keys.verification_keys.iter().for_each(|vk| {
        jwks.keys.push(vk.public_key.clone());
    });

    let mut headers = HeaderMap::new();
    let mut max_age = keys.next_rotation - Utc::now().timestamp();
    if max_age < 120 {
        max_age = 120
    }
    headers.insert(
        header::CACHE_CONTROL,
        format!("max-age={}, must-revalidate", max_age)
            .parse()
            .map_err(errors::any)?,
    );
    Ok((headers, Json(jwks)))
}

async fn auth_handler(
    app: AppState,
    Valid(mut auth_request): Valid<AuthRequest>,
) -> Result<(StatusCode, HeaderMap)> {
    let login_uri = auth(&app.store.connector, &mut auth_request).await?;
    // redirect to EU login uri
    let mut headers = HeaderMap::new();
    headers.insert(header::LOCATION, login_uri.parse().map_err(errors::any)?);
    Ok((StatusCode::FOUND, headers))
}

#[derive(Deserialize)]
struct GrantOpts {
    grant_type: String,
}

async fn token_handler(
    app: AppState,
    TypedHeader(info): TypedHeader<Authorization<Basic>>,
    RawForm(bytes): RawForm,
) -> Result<(HeaderMap, Json<token::TokenResponse>)> {
    let grant_opts: GrantOpts =
        serde_urlencoded::from_bytes(&bytes).map_err(errors::any)?;
    let res = match grant_opts.grant_type.as_str() {
        token::GRANT_TYPE_AUTHORIZATION_CODE => {
            let client_info = token::get_client_and_valid(
                &app.store.client,
                info.username(),
                info.password(),
            )
            .await?;
            let opts: token::code::CodeGrantOpts =
                serde_urlencoded::from_bytes(&bytes).map_err(errors::any)?;
            let cg = token::code::CodeGrant {
                auth_store: &app.store.auth_code,
                connector_store: &app.store.connector,
                token_creator: &app.access_token,
                refresh_token_store: &app.store.refresh,
                offline_session_store: &app.store.offline_session,
                user_store: &app.store.user,
            };
            cg.grant(&client_info, &opts).await?
        }
        token::GRANT_TYPE_REFRESH_TOKEN => {
            let client_info = token::get_client_and_valid(
                &app.store.client,
                info.username(),
                info.password(),
            )
            .await?;
            let opts: token::refresh::RefreshGrantOpts =
                serde_urlencoded::from_bytes(&bytes).map_err(errors::any)?;
            let rg = token::refresh::RefreshGrant {
                refresh_store: &app.store.refresh,
                connector_store: &app.store.connector,
                token_creator: &app.access_token,
                offline_session_store: &app.store.offline_session,
                user_store: &app.store.user,
                absolute_lifetime: Duration::seconds(
                    app.config.absolute_lifetime,
                ),
                valid_if_not_used_for: Duration::seconds(
                    app.config.valid_if_not_used_for,
                ),
                reuse_interval: Duration::seconds(app.config.reuse_interval),
                rotate_refresh_tokens: app.config.rotate_refresh_tokens,
            };
            rg.grant(&client_info, &opts).await?
        }
        token::GRANT_TYPE_PASSWORD => {
            let client_info = token::get_client_and_valid(
                &app.store.client,
                info.username(),
                info.password(),
            )
            .await?;
            let opts: token::password::PasswordGrantOpts =
                serde_urlencoded::from_bytes(&bytes).map_err(errors::any)?;
            let pg = token::password::PasswordGrant {
                client_store: &app.store.client,
                connector_store: &app.store.connector,
                token_creator: &app.access_token,
                refresh_token_store: &app.store.refresh,
                offline_session_store: &app.store.offline_session,
                user_store: &app.store.user,
            };
            pg.grant(&client_info, &opts, &app.config.password_connector)
                .await?
        }
        _ => {
            return Err(errors::bad_request(
                format!("unknown grant type: {}", grant_opts.grant_type)
                    .as_str(),
            ))
        }
    };
    let mut headers = HeaderMap::new();
    headers.insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
    headers.insert(header::PRAGMA, "no-cache".parse().unwrap());
    Ok((headers, Json(res)))
}

async fn userinfo(
    app: AppState,
    Valid(mut auth_request): Valid<AuthRequest>,
) -> Result<(StatusCode, HeaderMap)> {
    let login_uri = auth(&app.store.connector, &mut auth_request).await?;
    // redirect to EU login uri
    let mut headers = HeaderMap::new();
    headers.insert(header::LOCATION, login_uri.parse().map_err(errors::any)?);
    Ok((StatusCode::FOUND, headers))
}
