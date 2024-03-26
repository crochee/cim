pub mod password;
mod tokenx;

use async_trait::async_trait;
use mockall::automock;
use serde::{Deserialize, Serialize};

use slo::Result;

pub use tokenx::AccessToken;

#[automock]
#[async_trait]
pub trait Token {
    async fn token(
        &self,
        claims: &Claims,
        opts: &TokenOpts,
    ) -> Result<(String, i64)>;
    async fn verify(&self, token: &str) -> Result<Claims>;
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TokenClaims {
    pub aud: String, // Optional. Audience
    pub exp: i64, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    pub iat: i64, // Optional. Issued at (as UTC timestamp)
    pub iss: String, // Optional. Issuer
    pub sub: String, // Optional. Subject (whom token refers to)

    pub nonce: String,
    pub access_token_hash: String,
    pub code_hash: String,
    pub email: Option<String>,
    pub email_verified: bool,
    pub mobile: Option<String>,
    pub name: String,
    pub preferred_username: String,
}

#[derive(Debug)]
pub struct TokenOpts {
    pub scopes: Vec<String>,
    pub nonce: String,
    pub access_token: Option<String>,
    pub code: Option<String>,
    pub aud: String,
    pub issuer_url: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<i64>,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    pub scopes: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub user_id: String,
    pub username: String,
    pub preferred_username: String,
    pub email: Option<String>,
    pub email_verified: bool,
    pub mobile: Option<String>,
    pub exp: Option<i64>,
}

pub const GRANT_TYPE_AUTHORIZATION_CODE: &str = "authorization_code";
pub const GRANT_TYPE_REFRESH_TOKEN: &str = "refresh_token";
pub const GRANT_TYPE_IMPLICIT: &str = "implicit";
pub const GRANT_TYPE_PASSWORD: &str = "password";
