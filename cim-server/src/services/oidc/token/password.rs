use serde::Deserialize;

use cim_slo::{errors, Result};
use cim_storage::{
    client::{Client, ClientStore},
    connector::ConnectorStore,
    offlinesession::OfflineSessionStore,
    refresh::RefreshTokenStore,
    users,
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

pub struct PasswordGrant<'a, C, S, T, R, O, U> {
    pub client_store: &'a C,
    pub connector_store: &'a S,
    pub token_creator: &'a T,
    pub refresh_token_store: &'a R,
    pub offline_session_store: &'a O,
    pub user_store: &'a U,
}

impl<'a, C, S, T, R, O, U> PasswordGrant<'a, C, S, T, R, O, U>
where
    C: ClientStore,
    S: ConnectorStore,
    T: token::Token,
    R: RefreshTokenStore,
    O: OfflineSessionStore,
    U: users::UserStore + Send + Sync + Clone + 'static,
{
    pub async fn grant(
        &self,
        client_value: &Client,
        opts: &PasswordGrantOpts,
        password_conn: &str,
    ) -> Result<token::TokenResponse> {
        let scopes: Vec<String> = opts
            .scope
            .split_whitespace()
            .map(|x| x.to_owned())
            .collect();
        valid_scope(self.client_store, &client_value.id, &scopes).await?;
        let connector =
            get_connector(self.connector_store, password_conn).await?;
        let conn = match open_connector(self.user_store, &connector)? {
            Connector::Password(conn) => conn,
            _ => return Err(errors::bad_request("unsupported connector type")),
        };
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
            claim: identity.claim.clone(),
            ..Default::default()
        };
        let (access_token, _) = self.token_creator.token(&claims).await?;
        claims.access_token = Some(access_token.clone());
        let (id_token, expires_in) = self.token_creator.token(&claims).await?;

        let mut refresh_token_value = None;
        if conn.refresh_enabled() {
            let rt = token::RefreshTokenHandler {
                refresh_token_store: self.refresh_token_store,
                offline_session_store: self.offline_session_store,
            };
            refresh_token_value = rt
                .handle(
                    scopes.clone(),
                    &client_value.id,
                    &opts.nonce,
                    &identity.claim,
                    &connector.id,
                    identity.connector_data.clone(),
                )
                .await?;
        }
        Ok(token::TokenResponse {
            access_token,
            token_type: "bearer".to_owned(),
            expires_in: Some(expires_in),
            refresh_token: refresh_token_value,
            id_token: Some(id_token),
            scopes: Some(scopes),
        })
    }
}
