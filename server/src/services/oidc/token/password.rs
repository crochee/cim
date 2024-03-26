use constant_time_eq::constant_time_eq;
use serde::Deserialize;
use slo::{errors, Result};
use storage::{
    client::{Client, ClientStore},
    connector::ConnectorStore,
};
use validator::Validate;

use crate::services::oidc::{
    connect, get_connector, open_connector, token, valid_scope, Connector,
};

pub async fn get_client_and_valid<C: ClientStore>(
    client_store: &C,
    client_id: &str,
    client_secret: &str,
) -> Result<Client> {
    let client = client_store.get_client(client_id, None).await?;
    if !constant_time_eq(client.secret.as_bytes(), client_secret.as_bytes()) {
        return Err(errors::unauthorized());
    }
    Ok(client)
}

#[derive(Debug, Deserialize, Validate)]
pub struct PasswordGrantOpts {
    pub scope: String,
    pub nonce: String,
    pub username: String,
    pub password: String,
}

pub async fn password_grant<
    C: ClientStore,
    CS: ConnectorStore,
    T: token::Token,
>(
    client_store: &C,
    connector_store: &CS,
    token_creator: &T,
    client_value: &Client,
    opts: &PasswordGrantOpts,
) -> Result<token::TokenResponse> {
    let scopes: Vec<String> = opts
        .scope
        .split_whitespace()
        .map(|x| x.to_owned())
        .collect();
    valid_scope(client_store, &client_value.id, &scopes).await?;
    // TODO: system give id
    let connector = get_connector(connector_store, "").await?;
    match open_connector(&connector)? {
        Connector::Password(conn) => {
            let identity = conn
                .login(
                    &connect::parse_scopes(&scopes),
                    &connect::Info {
                        subject: opts.username.clone(),
                        password: opts.password.clone(),
                    },
                )
                .await?;
            let claims = token::Claims {
                user_id: identity.user_id,
                username: identity.username,
                preferred_username: identity.preferred_username,
                email: identity.email,
                email_verified: identity.email_verified,
                mobile: identity.mobile,
                exp: None,
            };
            // TODO: add fill token
            let mut token_opts = token::TokenOpts {
                scopes: scopes.clone(),
                nonce: opts.nonce.clone(),
                access_token: None,
                code: Some("sjhdkf".to_owned()),
                aud: "IO".to_owned(),
                issuer_url: "http://127.0.0.1:80".to_owned(),
            };
            let (access_token, _) =
                token_creator.token(&claims, &token_opts).await?;
            token_opts.access_token = Some(access_token.clone());

            let (id_token, expires_in) =
                token_creator.token(&claims, &token_opts).await?;
            //TODO: add refresh token
            Ok(token::TokenResponse {
                access_token,
                token_type: "bearer".to_owned(),
                expires_in: Some(expires_in),
                refresh_token: None,
                id_token: Some(id_token),
                scopes: Some(scopes),
            })
        }
        _ => Err(errors::bad_request("unsupported connector type")),
    }
}
