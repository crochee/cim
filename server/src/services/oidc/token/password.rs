use serde::Deserialize;
use slo::{errors, Result};
use storage::{
    client::{Client, ClientStore},
    connector::ConnectorStore,
};

use crate::services::oidc::{
    connect, get_connector, open_connector, token, valid_scope, Connector,
};

#[derive(Debug, Deserialize)]
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

            let mut claims = token::Claims {
                claim: identity.claim,
                ..Default::default()
            };
            let (access_token, _) = token_creator.token(&claims).await?;
            claims.access_token = Some(access_token.clone());
            let (id_token, expires_in) = token_creator.token(&claims).await?;
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
