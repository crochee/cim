pub mod connect;
pub mod key;
pub mod token;

use std::collections::HashMap;

use chrono::Utc;
use rand::Rng;
use serde::Deserialize;
use validator::Validate;

use cim_core::{Code, Result};

use crate::{
    models::{claim::Claims, provider::Provider},
    store::Store,
    AppState,
};

use self::{
    connect::{parse_scopes, Info, PasswordConnector},
    token::{Token, TokenOpts, TokenResponse},
};

#[derive(Debug, Deserialize, Validate)]
pub struct AuthReq {
    pub response_type: String,
    pub client_id: String,
    pub state: Option<String>,
    pub redirect_uri: String,
    pub scope: String,
}

#[derive(Debug, Deserialize)]
pub struct TokenReq {
    pub grant_type: String,
    pub client_id: String,
    pub state: Option<String>,
    pub redirect_uri: String,
    pub scope: String,
}

pub async fn password_grant_token(
    app: &AppState,
    body: &HashMap<String, String>,
    f: (String, String),
) -> Result<TokenResponse> {
    let (client_id, _) = &f;
    let provider = match app.store.get_provider(client_id).await {
        Ok(v) => v,
        Err(err) => {
            if err.eq(&Code::not_found("")) {
                return Err(Code::Unauthorized.with());
            }
            return Err(err);
        }
    };
    let connector = connect::UserIDPassword::new(app.store.clone());
    let token_handler =
        token::AccessToken::new(app.key_rotator.clone(), 30 * 60);

    password_grant(&provider, &token_handler, &connector, body, &f).await
}

async fn password_grant<T: Token, C: PasswordConnector>(
    provider: &Provider,
    token_creator: &T,
    connector: &C,
    body: &HashMap<String, String>,
    f: &(String, String),
) -> Result<TokenResponse> {
    let (client_id, client_secret) = f;
    if provider.secret.eq(client_secret) {
        return Err(Code::Unauthorized.with());
    }
    let nonce = body.get("nonce").unwrap_or(&"".to_owned()).to_owned();
    let default_scope = String::new();
    let scopes: Vec<String> = body
        .get("scope")
        .unwrap_or(&default_scope)
        .to_owned()
        .split_whitespace()
        .map(|x| x.to_owned())
        .collect();

    let mut has_open_id_scope = false;
    for scope in &scopes {
        if scope.eq("openid") {
            has_open_id_scope = true;
        }
    }
    if !has_open_id_scope {
        return Err(Code::bad_request(
            r#"Missing required scope(s) ["openid"]."#,
        ));
    }

    let username = body.get("username").unwrap_or(&"".to_owned()).to_owned();
    let password = body.get("password").unwrap_or(&"".to_owned()).to_owned();

    let (identity, ok) = connector
        .login(
            &parse_scopes(&scopes),
            &Info {
                subject: username,
                password,
            },
        )
        .await?;

    if !ok {
        return Err(Code::Unauthorized.with());
    }

    let claims = Claims {
        user_id: identity.user_id,
        username: identity.username,
        preferred_username: identity.preferred_username,
        email: identity.email,
        email_verified: identity.email_verified,
        mobile: identity.mobile,
        exp: None,
    };
    let access_token_pad = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(255)
        .map(char::from)
        .collect::<String>();
    let mut token_opts = TokenOpts {
        scopes: scopes.clone(),
        nonce,
        access_token: Some(access_token_pad),
        code: None,
        conn_id: client_id.to_string(),
        issuer_url: provider.redirect_url.to_string(),
    };
    let (access_token, _) = token_creator.token(&claims, &token_opts).await?;

    token_opts.access_token = Some(access_token.clone());
    let (id_token, exp) = token_creator.token(&claims, &token_opts).await?;

    let mut result = TokenResponse {
        access_token,
        token_type: "".to_owned(),
        expires_in: None,
        refresh_token: None,
        id_token: Some(id_token),
        scopes: Some(scopes),
    };
    if provider.refresh {
        result.refresh_token = Some("test".to_owned());
    }
    result.expires_in = Some(exp - Utc::now().timestamp());
    Ok(result)
}
