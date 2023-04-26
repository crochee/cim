use async_trait::async_trait;
use chrono::Utc;
use jsonwebtoken::{
    decode, decode_header, encode, Algorithm, DecodingKey, EncodingKey, Header,
    Validation,
};
use sha2::{Digest, Sha256};
use tracing::error;

use crate::{
    models::{claim::Claims, key::KeyValue},
    services::authentication::key::KeysStore,
};

use super::{Token, TokenClaims, TokenOpts};

use cim_core::{Code, Result};

pub struct AccessToken<T> {
    key_store: T,
    expire_sec: i64,
}

impl<T> AccessToken<T> {
    pub fn new(key_store: T, expire_sec: i64) -> Self {
        Self {
            key_store,
            expire_sec,
        }
    }
}

#[async_trait]
impl<T> Token for AccessToken<T>
where
    T: KeysStore,
{
    async fn token(
        &self,
        claims: &Claims,
        opts: &TokenOpts,
    ) -> Result<(String, i64)> {
        let keys = self.key_store.get().await?;
        let issued_at = Utc::now().timestamp();
        let exp = issued_at + self.expire_sec;

        let mut token_claims = TokenClaims {
            aud: opts.conn_id.clone(),
            exp,
            iat: issued_at,
            iss: opts.issuer_url.clone(),
            sub: claims.user_id.clone(),
            nonce: opts.nonce.clone(),
            email: claims.email.clone(),
            email_verified: claims.email_verified,
            name: claims.username.clone(),
            preferred_username: claims.preferred_username.clone(),
            ..Default::default()
        };

        if let Some(access_token) = &opts.access_token {
            token_claims.access_token_hash = format!(
                "{:?}",
                Sha256::new().chain_update(access_token).finalize()
            );
        };
        if let Some(code) = &opts.code {
            token_claims.code_hash =
                format!("{:?}", Sha256::new().chain_update(code).finalize());
        };
        let token = encode(
            &Header::new(Algorithm::HS256),
            &token_claims,
            &EncodingKey::from_secret(keys.signing_key.value.as_bytes()),
        )
        .map_err(Code::any)?;
        Ok((token, exp))
    }

    async fn verify(&self, token: &str) -> Result<Claims> {
        let header = decode_header(token).map_err(Code::any)?;
        let keys = self.key_store.get().await?;
        let mut keys_list =
            Vec::with_capacity(keys.verification_keys.len() + 1);
        keys_list.push(KeyValue {
            id: keys.signing_key.id,
            value: keys.signing_key.value,
            alg: keys.signing_key.alg,
        });
        for vk in &keys.verification_keys {
            keys_list.push(KeyValue {
                id: vk.value.id.clone(),
                value: vk.value.value.clone(),
                alg: vk.value.alg.clone(),
            });
        }
        for vk in keys_list.iter() {
            if let Some(kid) = &header.kid {
                if vk.id.eq(kid) {
                    continue;
                }
            }
            match decode::<TokenClaims>(
                token,
                &DecodingKey::from_secret(vk.value.as_bytes()),
                &Validation::default(),
            ) {
                Ok(v) => {
                    return Ok(Claims {
                        user_id: v.claims.sub,
                        username: v.claims.name,
                        preferred_username: v.claims.preferred_username,
                        email: v.claims.email,
                        email_verified: v.claims.email_verified,
                        mobile: v.claims.mobile,
                        exp: Some(v.claims.exp),
                    })
                }
                Err(err_value) => {
                    error!("{}", err_value);
                }
            }
        }
        Err(Code::forbidden("failed to verify id token signature"))
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use crate::{
        models::{
            claim::Claims,
            key::{KeyValue, Keys},
        },
        services::authentication::{
            key::MockKeysStore,
            token::{AccessToken, Token, TokenOpts},
        },
    };

    #[tokio::test]
    async fn token_test() {
        let mut key_store = MockKeysStore::new();
        key_store.expect_get().returning(|| {
            Ok(Keys {
                signing_key: KeyValue {
                    id: "".to_owned(),
                    value: "s".to_owned(),
                    alg: "HS256".to_owned(),
                },
                verification_keys: vec![],
                next_rotation: 0,
            })
        });
        let t = AccessToken::new(key_store, 30);
        let access_token = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(255)
            .map(char::from)
            .collect::<String>();
        let (token_str, exp) = t
            .token(
                &Claims {
                    user_id: "1".to_owned(),
                    username: "lee".to_owned(),
                    preferred_username: "crochee".to_owned(),
                    email: None,
                    email_verified: false,
                    mobile: None,
                    exp: None,
                },
                &TokenOpts {
                    scopes: vec![
                        "email".to_owned(),
                        "openid".to_owned(),
                        "profile".to_owned(),
                    ],
                    nonce: "hsjdkjfka".to_owned(),
                    access_token: Some(access_token),
                    code: Some("sjhdkf".to_owned()),
                    conn_id: "IO".to_owned(),
                    issuer_url: "http://127.0.0.1:80".to_owned(),
                },
            )
            .await
            .unwrap();
        println!("{} {}", exp, token_str);
        let claims = t.verify(&token_str).await.unwrap();
        println!("{:#?}", claims);
    }
}
