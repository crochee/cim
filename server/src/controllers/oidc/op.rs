use axum::{
    routing::get,
    {Json, Router},
};
use chrono::Utc;
use http::{header, HeaderMap, StatusCode};
use serde::Serialize;
use slo::{errors, Result};
use utoipa::ToSchema;

use storage::keys::*;

use crate::{
    services::oidc::auth::{auth, AuthRequest},
    valid::Valid,
};
use crate::{services::oidc::key, AppState};

pub fn new_router(state: AppState) -> Router {
    Router::new()
        // op common api
        .route("/.well-known/openid-configuration", get(discovery_handler))
        .route("/jwks", get(jwk_handler))
        // auth api
        .route("/auth", get(auth_handler))
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
    jwks.keys.reverse();

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
