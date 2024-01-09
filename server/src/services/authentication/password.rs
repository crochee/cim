use chrono::Utc;
use rand::Rng;

use serde::Deserialize;

use crate::{
    errors,
    models::{claim::Claims, provider::Provider},
    store::Store,
    AppState, Result,
};

use super::{
    connect::{self, parse_scopes, Info, PasswordConnector},
    token::{self, Token, TokenOpts, TokenResponse},
};

#[derive(Debug, Deserialize)]
pub struct PasswordGrantOpts {
    pub grant_type: String,
    pub username: String,
    pub password: String,
    pub scope: String,
    pub audience: String,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub nonce: Option<String>,
}

pub async fn password_grant_token(
    app: &AppState,
    body: &PasswordGrantOpts,
) -> Result<TokenResponse> {
    let provider = match app
        .store
        .get_provider(&body.client_id.clone().unwrap_or_default())
        .await
    {
        Ok(v) => v,
        Err(err) => {
            if err.eq(&errors::not_found("")) {
                return Err(errors::unauthorized());
            }
            return Err(err);
        }
    };
    let connector = connect::UserIDPassword::new(app.store.clone());
    let token_handler =
        token::AccessToken::new(app.key_rotator.clone(), 30 * 60);

    password_grant(&provider, &token_handler, &connector, body).await
}

async fn password_grant<T: Token, C: PasswordConnector>(
    provider: &Provider,
    token_creator: &T,
    connector: &C,
    body: &PasswordGrantOpts,
) -> Result<TokenResponse> {
    if provider
        .secret
        .ne(&body.client_secret.clone().unwrap_or_default())
    {
        return Err(errors::unauthorized());
    }
    let scopes: Vec<String> = body
        .scope
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
        return Err(errors::bad_request(
            r#"Missing required scope(s) ["openid"]."#,
        ));
    }

    let (identity, ok) = connector
        .login(
            &parse_scopes(&scopes),
            &Info {
                subject: body.username.clone(),
                password: body.password.clone(),
            },
        )
        .await?;

    if !ok {
        return Err(errors::unauthorized());
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
        nonce: body.nonce.clone().unwrap_or_default(),
        access_token: Some(access_token_pad),
        code: None,
        conn_id: body.client_id.clone().unwrap_or_default(),
        issuer_url: provider.redirect_url.to_string(),
    };
    let (access_token, _) = token_creator.token(&claims, &token_opts).await?;

    token_opts.access_token = Some(access_token.clone());
    let (id_token, exp) = token_creator.token(&claims, &token_opts).await?;

    let mut result = TokenResponse {
        access_token,
        token_type: "bearer".to_owned(),
        expires_in: None,
        refresh_token: None,
        id_token: Some(id_token),
        scopes: Some(scopes),
    };
    if provider.refresh {
        // TODO pad detail refresh token
        result.refresh_token = Some("test".to_owned());
    }
    result.expires_in = Some(exp - Utc::now().timestamp());
    Ok(result)
}
