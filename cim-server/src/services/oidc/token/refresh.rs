use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;

use cim_slo::{errors, Result};
use cim_storage::{
    client, connector, offlinesession, refresh_token, user, Interface, List,
    WatchInterface,
};

use crate::services::oidc::{
    connect::{self, parse_scopes},
    open_connector, token, valid_scope,
};

#[derive(Debug, Deserialize)]
pub struct RefreshGrantOpts {
    pub client_id: String,
    pub client_secret: String,

    pub refresh_token: String,
    pub scope: String,
}

pub struct RefreshGrant<'a, S, R, C, T, O, U> {
    pub client_store: &'a S,
    pub refresh_store: &'a R,
    pub connector_store: &'a C,
    pub token_creator: &'a T,
    pub offline_session_store: &'a O,
    pub user_store: &'a U,
    pub absolute_lifetime: Duration,
    pub valid_if_not_used_for: Duration,
    pub reuse_interval: Duration,
    pub rotate_refresh_tokens: bool,
}

impl<S, R, C, T, O, U> RefreshGrant<'_, S, R, C, T, O, U>
where
    S: Interface<T = client::Client>,
    R: Interface<T = refresh_token::RefreshToken>,
    C: Interface<T = connector::Connector>,
    T: token::Token,
    O: Interface<
        T = offlinesession::OfflineSession,
        L = offlinesession::ListParams,
    >,
    U: WatchInterface<T = user::User> + Send + Sync + Clone + 'static,
{
    pub fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }

    pub fn completely_expired(&self, created_at: DateTime<Utc>) -> bool {
        if self.absolute_lifetime.is_zero() {
            return false;
        }
        self.now() > (created_at + self.absolute_lifetime)
    }
    pub fn expired_because_unused(&self, last_used: DateTime<Utc>) -> bool {
        if self.valid_if_not_used_for.is_zero() {
            return false;
        }
        self.now() > (last_used + self.valid_if_not_used_for)
    }
    pub fn allow_reused(&self, last_used: DateTime<Utc>) -> bool {
        if self.reuse_interval.is_zero() {
            return false;
        }
        self.now() < (last_used + self.reuse_interval)
    }
    pub async fn grant(
        &self,
        client_info: &client::Client,
        opts: &RefreshGrantOpts,
    ) -> Result<token::TokenResponse> {
        let mut claim_refresh_token: token::ClaimRefreshToken =
            serde_json::from_str(&opts.refresh_token).map_err(errors::any)?;
        let mut refresh_token = refresh_token::RefreshToken {
            id: claim_refresh_token.refresh_id.clone(),
            ..Default::default()
        };
        self.refresh_store.get(&mut refresh_token).await?;
        if !refresh_token.client_id.eq(&client_info.id) {
            return Err(errors::bad_request(
                "refresh token does not belong to this client",
            ));
        }
        if !refresh_token.token.eq(&claim_refresh_token.token) {
            return Err(errors::bad_request("invalid refresh token"));
        }
        if self.completely_expired(refresh_token.created_at.and_utc()) {
            return Err(errors::bad_request("refresh token expired"));
        }
        if self.expired_because_unused(refresh_token.last_used_at.and_utc()) {
            return Err(errors::bad_request(
                "refresh token expired because it was not used in time",
            ));
        }
        let mut connector_value = connector::Connector {
            id: refresh_token.connector_id.clone(),
            ..Default::default()
        };
        self.connector_store.get(&mut connector_value).await?;

        let mut sessions = List::default();
        self.offline_session_store
            .list(
                &offlinesession::ListParams {
                    user_id: Some(refresh_token.claim.sub.clone()),
                    conn_id: Some(refresh_token.connector_id.clone()),
                    pagination: cim_storage::Pagination {
                        count_disable: true,
                        ..Default::default()
                    },
                },
                &mut sessions,
            )
            .await?;
        if sessions.data.is_empty() {
            return Err(errors::not_found("Offline session not found"));
        }
        let mut session = sessions.data.remove(0);

        match &refresh_token.connector_data {
            Some(v) => connector_value.connector_data = Some(v.clone()),
            None => {
                connector_value.connector_data = session.connector_data.clone()
            }
        }

        let scopes: Vec<String> = opts
            .scope
            .split_whitespace()
            .map(|x| x.to_owned())
            .collect();
        let mut unauthorized_scopes = Vec::new();
        for scope in &scopes {
            if !refresh_token.scopes.contains(&scope.to_string()) {
                unauthorized_scopes.push(scope.to_string());
            }
        }
        if unauthorized_scopes.is_empty() {
            return Err(errors::bad_request(
                format!(
                    "Requested scopes contain unauthorized scope(s): {:?}",
                    unauthorized_scopes,
                )
                .as_str(),
            ));
        }

        let audience =
            valid_scope(self.client_store, &client_info.id, &scopes).await?;
        let aud = match audience.len() {
            0 => "".to_string(),
            1 => audience[0].clone(),
            _ => serde_json::to_string(&audience).map_err(errors::any)?,
        };
        let identity = self
            .update_refresh_token(
                &mut claim_refresh_token,
                &mut refresh_token,
                &connector_value,
                &mut session,
                &scopes,
            )
            .await?;
        let refresh_token_value =
            serde_json::to_string(&claim_refresh_token).map_err(errors::any)?;

        let mut claims = token::Claims {
            claim: identity.claim.clone(),
            nonce: refresh_token.nonce.clone(),
            aud,
            ..Default::default()
        };
        let (access_token, _) = self.token_creator.token(&claims).await?;
        claims.access_token = Some(access_token.clone());
        let (id_token, expires_in) = self.token_creator.token(&claims).await?;

        Ok(token::TokenResponse {
            access_token,
            token_type: "bearer".to_owned(),
            expires_in: Some(expires_in),
            refresh_token: Some(refresh_token_value),
            id_token: Some(id_token),
            scopes: Some(scopes),
        })
    }

    async fn update_refresh_token(
        &self,
        claim_refresh_token: &mut token::ClaimRefreshToken,
        refresh_token: &mut refresh_token::RefreshToken,
        connector_value: &connector::Connector,
        offline_session: &mut offlinesession::OfflineSession,
        scopes: &Vec<String>,
    ) -> Result<connect::Identity> {
        let mut ident = connect::Identity {
            claim: refresh_token.claim.clone(),
            connector_data: connector_value.connector_data.clone(),
        };
        let mut last_used = self.now();
        let reused_allow =
            self.allow_reused(refresh_token.last_used_at.and_utc());
        if !self.rotate_refresh_tokens && reused_allow {
            refresh_token.connector_data = None;
        } else if self.rotate_refresh_tokens && reused_allow {
            if !refresh_token.token.eq(&claim_refresh_token.token)
                && !refresh_token.obsolete_token.eq(&claim_refresh_token.token)
            {
                return Err(errors::bad_request(
                    "refresh token has already been used",
                ));
            }
            if refresh_token.obsolete_token.eq(&claim_refresh_token.token) {
                claim_refresh_token.token = refresh_token.token.clone();
            }
            last_used = refresh_token.last_used_at.and_utc();
            refresh_token.connector_data = None;
        } else if self.rotate_refresh_tokens && !reused_allow {
            if !refresh_token.token.eq(&claim_refresh_token.token) {
                return Err(errors::bad_request(
                    "refresh token has already been used",
                ));
            }
            refresh_token.obsolete_token = refresh_token.token.clone();
            claim_refresh_token.token = uuid::Uuid::new_v4().to_string();
        }
        refresh_token.token = claim_refresh_token.token.clone();
        refresh_token.last_used_at = last_used.naive_utc();
        refresh_token.connector_data = None;
        let connector_impl =
            open_connector(self.user_store, Some(connector_value))?;
        if connector_impl.support_refresh() {
            ident = connector_impl
                .refresh(&parse_scopes(scopes), &ident)
                .await?;
        }
        refresh_token.claim = ident.claim.clone();
        self.refresh_store.put(refresh_token).await?;

        if let Some(offline_session_value) =
            offline_session.refresh.get_mut(&refresh_token.client_id)
        {
            if !offline_session_value.id.eq(&refresh_token.id) {
                return Err(errors::bad_request("refresh token invalid"));
            }
            offline_session_value.last_used_at = last_used.naive_utc();
            if let Some(connector_data) = &ident.connector_data {
                offline_session.connector_data = Some(connector_data.clone());
            }
        }

        self.offline_session_store.put(offline_session).await?;
        Ok(ident)
    }
}
