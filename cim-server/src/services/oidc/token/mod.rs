pub mod code;
pub mod password;
pub mod refresh;
mod tokenx;

use async_trait::async_trait;
use chrono::Utc;
use constant_time_eq::constant_time_eq;
use mockall::automock;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

use cim_slo::{errors, next_id, Result};
use cim_storage::{
    client, offlinesession, refresh_token, Claim, Interface, List,
};

pub use tokenx::AccessToken;

#[automock]
#[async_trait]
pub trait Token {
    async fn token(&self, claims: &Claims) -> Result<(String, i64)>;
    async fn verify(&self, token: &str) -> Result<Claims>;
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Claims {
    pub aud: String, // Optional. Audience
    pub exp: i64, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    pub nbf: i64, // Optional. Not Before (as UTC timestamp)
    pub iss: String, // Optional. Issuer

    pub nonce: String,
    pub access_token: Option<String>,

    #[serde(flatten)]
    pub claim: Claim,
}

#[derive(Debug, Clone, Deserialize, Default, Serialize)]
pub struct ClaimRefreshToken {
    pub refresh_id: String,
    pub token: String,
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

pub const GRANT_TYPE_AUTHORIZATION_CODE: &str = "authorization_code";
pub const GRANT_TYPE_REFRESH_TOKEN: &str = "refresh_token";
pub const GRANT_TYPE_PASSWORD: &str = "password";

pub async fn get_client_and_valid<C: Interface<T = client::Client>>(
    client_store: &C,
    client_id: &str,
    client_secret: &str,
) -> Result<client::Client> {
    let mut client = client::Client::default();
    client_store.get(client_id, &mut client).await?;

    if !constant_time_eq(client.secret.as_bytes(), client_secret.as_bytes()) {
        return Err(errors::forbidden("invalid client secret"));
    }
    Ok(client)
}

pub struct RefreshTokenHandler<'a, R, O> {
    pub refresh_token_store: &'a R,
    pub offline_session_store: &'a O,
}

impl<'a, R, O> RefreshTokenHandler<'a, R, O>
where
    R: Interface<T = refresh_token::RefreshToken>,
    O: Interface<
        T = offlinesession::OfflineSession,
        L = offlinesession::ListParams,
    >,
{
    pub async fn handle(
        &self,
        scopes: Vec<String>,
        client_id: &str,
        nonce: &str,
        claim: &Claim,
        connector_id: &str,
        connector_data: Option<Box<RawValue>>,
    ) -> Result<Option<String>> {
        let mut refresh_token_value = None;
        if scopes.contains(&String::from("offline_access")) {
            let refresh_token = refresh_token::RefreshToken {
                id: next_id().map_err(errors::any)?.to_string(),
                client_id: client_id.to_string(),
                scopes: scopes.clone(),
                nonce: nonce.to_string(),
                token: uuid::Uuid::new_v4().to_string(),
                claim: claim.clone(),
                connector_id: connector_id.to_string(),
                connector_data: connector_data.clone(),
                last_used_at: Utc::now().naive_utc(),
                ..Default::default()
            };
            refresh_token_value = Some(
                serde_json::to_string(&ClaimRefreshToken {
                    refresh_id: refresh_token.id.clone(),
                    token: refresh_token.token.clone(),
                })
                .map_err(errors::any)?,
            );

            self.refresh_token_store.put(&refresh_token, 0).await?;
            match self.handle_offline(&refresh_token, &refresh_token.id).await {
                Ok(Some(token_ref_id)) => {
                    self.refresh_token_store.delete(&token_ref_id).await?;
                }
                Err(err) => {
                    tracing::error!(
                        "failed to handle offline session: {}",
                        err
                    );
                    self.refresh_token_store.delete(&refresh_token.id).await?;
                    return Err(err);
                }
                _ => {}
            }
        }

        Ok(refresh_token_value)
    }
    async fn handle_offline(
        &self,
        refresh_token: &refresh_token::RefreshToken,
        id: &str,
    ) -> Result<Option<String>> {
        let token_ref = offlinesession::RefreshTokenRef {
            id: id.to_owned(),
            client_id: refresh_token.client_id.clone(),
            created_at: refresh_token.created_at,
            last_used_at: refresh_token.last_used_at,
        };
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
            let mut offline_session = offlinesession::OfflineSession {
                id: next_id().map_err(errors::any)?.to_string(),
                user_id: refresh_token.claim.sub.clone(),
                conn_id: refresh_token.connector_id.clone(),
                connector_data: refresh_token.connector_data.clone(),
                ..Default::default()
            };
            offline_session
                .refresh
                .insert(token_ref.client_id.clone(), token_ref);

            self.offline_session_store.put(&offline_session, 0).await?;
        } else {
            let mut session = sessions.data.remove(0);
            if let Some(old_session) =
                session.refresh.get_mut(&token_ref.client_id)
            {
                return Ok(Some(old_session.id.clone()));
            }
            session
                .refresh
                .insert(token_ref.client_id.clone(), token_ref);
            session.connector_data = refresh_token.connector_data.clone();
            self.offline_session_store.put(&session, 0).await?;
        }

        Ok(None)
    }
}
