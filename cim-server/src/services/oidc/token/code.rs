use base64::engine::{general_purpose, Engine};
use chrono::Utc;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use cim_slo::{errors, Result};
use cim_storage::{
    authcode, client, connector, offlinesession, refresh_token, user, Interface,
};

use crate::services::oidc::{self, get_connector, open_connector, token};

#[derive(Debug, Deserialize)]
pub struct CodeGrantOpts {
    pub client_id: String,
    pub client_secret: String,

    pub code: String,
    pub redirect_uri: String,
    pub code_verifier: Option<String>,
}

pub struct CodeGrant<'a, A, S, T, R, O, U> {
    pub auth_store: &'a A,
    pub connector_store: &'a S,
    pub token_creator: &'a T,
    pub refresh_token_store: &'a R,
    pub offline_session_store: &'a O,
    pub user_store: &'a U,
}

impl<'a, A, S, T, R, O, U> CodeGrant<'a, A, S, T, R, O, U>
where
    A: Interface<T = authcode::AuthCode>,
    S: Interface<T = connector::Connector>,
    T: token::Token,
    R: Interface<T = refresh_token::RefreshToken>,
    O: Interface<
        T = offlinesession::OfflineSession,
        L = offlinesession::ListParams,
    >,
    U: Interface<T = user::User> + Send + Sync + Clone + 'static,
{
    pub async fn grant(
        &self,
        client_value: &client::Client,
        opts: &CodeGrantOpts,
    ) -> Result<token::TokenResponse> {
        if opts.code.is_empty() {
            return Err(errors::bad_request("code is empty"));
        }
        let mut auth_code = authcode::AuthCode::default();
        self.auth_store.get(&opts.code, &mut auth_code).await?;

        if Utc::now().timestamp() > auth_code.expiry {
            return Err(errors::bad_request("code is expired"));
        }
        if !auth_code.client_id.eq(client_value.id.as_str()) {
            return Err(errors::bad_request("code is not belong to client"));
        }
        // code_challenge check
        let code_verifier = opts.code_verifier.clone().unwrap_or_default();
        if code_verifier.is_empty() && !auth_code.code_challenge.is_empty() {
            return Err(errors::bad_request(
                "Expecting parameter code_verifier in PKCE flow.",
            ));
        }
        if !code_verifier.is_empty() && auth_code.code_challenge.is_empty() {
            return Err(errors::bad_request(
                "No PKCE flow started. Cannot check code_verifier.",
            ));
        }
        let code_challenge = calculate_code_challenge(
            &code_verifier,
            &auth_code.code_challenge_method,
        )?;
        if !code_challenge.eq(&auth_code.code_challenge) {
            return Err(errors::bad_request("code_challenge is not match"));
        }
        if auth_code.redirect_uri != opts.redirect_uri {
            return Err(errors::bad_request("redirect_uri is not match"));
        }
        let mut claims = token::Claims {
            claim: auth_code.claim.clone(),
            nonce: auth_code.nonce.clone(),
            aud: auth_code.client_id.clone(),
            ..Default::default()
        };

        let (access_token, _) = self.token_creator.token(&claims).await?;
        claims.access_token = Some(access_token.clone());

        let (id_token, expires_in) = self.token_creator.token(&claims).await?;
        self.auth_store.delete(&opts.code).await?;

        let connector =
            get_connector(self.connector_store, &auth_code.connector_id)
                .await?;

        let mut refresh_token_value = None;
        if let oidc::Connector::Password(conn) =
            open_connector(self.user_store, &connector)?
        {
            if conn.refresh_enabled() {
                let rt = token::RefreshTokenHandler {
                    refresh_token_store: self.refresh_token_store,
                    offline_session_store: self.offline_session_store,
                };
                refresh_token_value = rt
                    .handle(
                        auth_code.scopes.clone(),
                        &client_value.id,
                        &auth_code.nonce,
                        &auth_code.claim,
                        &connector.id,
                        auth_code.connector_data.clone(),
                    )
                    .await?;
            }
        }
        Ok(token::TokenResponse {
            access_token,
            token_type: "bearer".to_owned(),
            expires_in: Some(expires_in),
            refresh_token: refresh_token_value,
            id_token: Some(id_token),
            scopes: Some(auth_code.scopes),
        })
    }
}

fn calculate_code_challenge(
    code_verifier: &str,
    code_challenge_method: &str,
) -> Result<String> {
    match code_challenge_method {
        oidc::CODE_CHALLENGE_METHOD_PLAIN => Ok(code_verifier.to_owned()),
        oidc::CODE_CHALLENGE_METHOD_S256 => {
            Ok(general_purpose::URL_SAFE_NO_PAD
                .encode(Sha256::new().chain_update(code_verifier).finalize()))
        }
        _ => Err(errors::bad_request("Invalid code_challenge_method value")),
    }
}
