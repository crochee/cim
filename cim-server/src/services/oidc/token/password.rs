use axum::extract::Request;
use axum_extra::headers::{authorization::Credentials, Authorization};
use http::header;
use serde::Deserialize;

use cim_slo::{errors, Result};
use cim_storage::{
    client, connector, offlinesession, refresh_token, user, Interface, List,
    Pagination, WatchInterface,
};

use crate::services::oidc::{open_connector, token, valid_scope};

#[derive(Debug, Deserialize)]
pub struct PasswordGrantOpts {
    pub client_id: String,
    pub client_secret: String,

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

impl<C, S, T, R, O, U> PasswordGrant<'_, C, S, T, R, O, U>
where
    C: Interface<T = client::Client>,
    S: Interface<T = connector::Connector, L = connector::ListParams>,
    T: token::Token,
    R: Interface<T = refresh_token::RefreshToken>,
    O: Interface<
        T = offlinesession::OfflineSession,
        L = offlinesession::ListParams,
    >,
    U: WatchInterface<T = user::User> + Send + Sync + Clone + 'static,
{
    pub async fn grant(
        &self,
        client_value: &client::Client,
        opts: &PasswordGrantOpts,
    ) -> Result<token::TokenResponse> {
        let scopes: Vec<String> = opts
            .scope
            .split_whitespace()
            .map(|x| x.to_owned())
            .collect();
        let audience =
            valid_scope(self.client_store, &client_value.id, &scopes).await?;
        let aud = match audience.len() {
            0 => "".to_string(),
            1 => audience[0].clone(),
            _ => serde_json::to_string(&audience).map_err(errors::any)?,
        };
        let connector_impl = open_connector(self.user_store, None)?;

        let mut req = Request::default();
        let headers = req.headers_mut();
        let Authorization(ab) =
            Authorization::basic(&opts.username, &opts.password);
        headers.insert(header::AUTHORIZATION, ab.encode());
        let identity = connector_impl.handle_callback(&scopes, req).await?;

        let mut claims = token::Claims {
            claim: identity.claim.clone(),
            nonce: opts.nonce.clone(),
            aud,
            ..Default::default()
        };
        let (access_token, _) = self.token_creator.token(&claims).await?;
        claims.access_token = Some(access_token.clone());
        let (id_token, expires_in) = self.token_creator.token(&claims).await?;

        let mut refresh_token_value = None;
        if connector_impl.support_refresh()
            && scopes.contains(&"offline_access".to_string())
        {
            let rt = token::RefreshTokenHandler {
                refresh_token_store: self.refresh_token_store,
                offline_session_store: self.offline_session_store,
            };
            let mut connectors = List::default();
            self.connector_store
                .list(
                    &connector::ListParams {
                        connector_type: Some("local".to_string()),
                        pagination: Pagination {
                            count_disable: true,
                            ..Default::default()
                        },
                    },
                    &mut connectors,
                )
                .await?;
            if connectors.data.is_empty() {
                return Err(errors::not_found("no connectors"));
            }
            refresh_token_value = Some(
                rt.handle(
                    scopes.clone(),
                    &client_value.id,
                    &opts.nonce,
                    &identity.claim,
                    &connectors.data[0].id,
                    identity.connector_data.clone(),
                )
                .await?,
            );
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
