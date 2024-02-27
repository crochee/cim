use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use utoipa::ToSchema;

use crate::AppState;

pub fn new_router(state: AppState) -> Router {
    Router::new()
        .route("/.well-known/openid-configuration", get(discovery_handler))
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
    pub claims_supported: Vec<String>,
}

async fn discovery_handler(app: AppState) -> Json<OpenIDConfiguration> {
    let issuer = format!("http://{}:{}", app.config.endpoint, app.config.port);
    Json(OpenIDConfiguration {
        issuer: issuer.to_string(),
        authorization_endpoint: format!("{issuer}/auth"),
        token_endpoint: format!("{issuer}/token"),
        jwks_uri: format!("{issuer}/certs"),
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
        response_types_supported: vec!["code".to_string()],
        scopes_supported: vec![
            "openid".to_string(),
            "profile".to_string(),
            "email".to_string(),
        ],
        token_endpoint_auth_methods_supported: vec![
            "client_secret_basic".to_string(),
            "client_secret_post".to_string(),
        ],
        claims_supported: vec![
            "aud".to_string(),
            "email".to_string(),
            "email_verified".to_string(),
            "exp".to_string(),
            "family_name".to_string(),
            "given_name".to_string(),
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
