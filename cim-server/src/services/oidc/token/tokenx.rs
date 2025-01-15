use std::collections::HashSet;

use async_trait::async_trait;
use base64::engine::{general_purpose, Engine};
use chrono::Utc;
use jsonwebkey as jwk;
use jsonwebtoken as jwt;
use sha2::{Digest, Sha256};
use tracing::error;

use cim_slo::{errors, Result};
use cim_storage::{key::Keys, Interface, List};

use super::{Claims, Token};

pub struct AccessToken<T> {
    key_store: T,
    expire_sec: i64,
    aud: HashSet<String>,
}

impl<T> AccessToken<T> {
    pub fn new(key_store: T, expire_sec: i64, aud: HashSet<String>) -> Self {
        Self {
            key_store,
            expire_sec,
            aud,
        }
    }

    fn now(&self) -> i64 {
        Utc::now().timestamp()
    }
}

#[async_trait]
impl<T> Token for AccessToken<T>
where
    T: Interface<T = Keys, L = ()> + Send,
{
    async fn token(&self, claims: &Claims) -> Result<(String, i64)> {
        let mut output: List<Keys> = List::default();
        self.key_store.list(&(), &mut output).await?;
        if output.data.is_empty() {
            return Err(errors::not_found("key not found"));
        }
        let keys = output.data.remove(0);

        let now = self.now();

        let mut token_claims = claims.clone();
        token_claims.exp = now + self.expire_sec;
        token_claims.nbf = now;

        if let Some(access_token) = &claims.access_token {
            token_claims.access_token = Some(
                general_purpose::STANDARD
                    .encode(Sha256::new_with_prefix(access_token).finalize()),
            );
        };
        let mut header = match keys.signing_key.algorithm {
            Some(v) => jwt::Header::new(match v {
                jwk::Algorithm::HS256 => jwt::Algorithm::HS256,
                jwk::Algorithm::ES256 => jwt::Algorithm::ES256,
                jwk::Algorithm::RS256 => jwt::Algorithm::RS256,
            }),
            None => jwt::Header::default(),
        };
        header.kid = keys.signing_key.key_id.clone();

        let token = jwt::encode(
            &header,
            &token_claims,
            &JwkKey(*keys.signing_key.key).to_encoding_key(),
        )
        .map_err(errors::any)?;
        Ok((token, token_claims.exp))
    }

    async fn verify(&self, token: &str) -> Result<Claims> {
        let mut output: List<Keys> = List::default();
        self.key_store.list(&(), &mut output).await?;
        if output.data.is_empty() {
            return Err(errors::not_found("key not found"));
        }
        let keys = output.data.remove(0);

        let header = jwt::decode_header(token).map_err(errors::any)?;

        for vk in keys.verification_keys.iter() {
            if header.kid.is_some() && !vk.public_key.key_id.eq(&header.kid) {
                continue;
            }
            let alg = match vk.public_key.algorithm {
                Some(v) => match v {
                    jwk::Algorithm::HS256 => jwt::Algorithm::HS256,
                    jwk::Algorithm::ES256 => jwt::Algorithm::ES256,
                    jwk::Algorithm::RS256 => jwt::Algorithm::RS256,
                },
                None => jwt::Algorithm::HS256,
            };
            let mut validation = jwt::Validation::new(alg);
            validation.validate_nbf = true;
            if self.aud.is_empty() {
                validation.validate_aud = false;
            } else {
                validation.aud = Some(self.aud.clone());
            }
            match jwt::decode::<Claims>(
                token,
                &JwkKey(*vk.public_key.key.clone()).to_decoding_key(),
                &validation,
            ) {
                Ok(v) => return Ok(v.claims),
                Err(err_value) => {
                    println!("{:?}", err_value);
                    error!("{}", err_value);
                }
            }
        }
        Err(errors::forbidden("failed to verify id token signature"))
    }
}

pub struct JwkKey<T>(pub T);

impl JwkKey<jwk::Key> {
    /// Returns an `EncodingKey` if the key is private.
    pub fn try_to_encoding_key(&self) -> Result<jwt::EncodingKey> {
        if !self.0.is_private() {
            return Err(errors::forbidden("key is not private"));
        }
        Ok(match &self.0 {
            jwk::Key::Symmetric { key } => {
                jwt::EncodingKey::from_secret(key.to_vec().as_slice())
            }
            // The following two conversion will not panic, as we've ensured that the keys
            // are private and tested that the successful output of `try_to_pem` is valid.
            jwk::Key::EC { .. } => jwt::EncodingKey::from_ec_pem(
                self.0.try_to_pem().map_err(errors::any)?.as_bytes(),
            )
            .map_err(errors::any)?,
            jwk::Key::RSA { .. } => {
                let pem = self.0.try_to_pem().map_err(errors::any)?;
                jwt::EncodingKey::from_rsa_pem(pem.as_bytes())
                    .map_err(errors::any)?
            }
        })
    }

    /// Unwrapping `try_to_encoding_key`. Panics if the key is public.
    pub fn to_encoding_key(&self) -> jwt::EncodingKey {
        self.try_to_encoding_key().unwrap()
    }

    pub fn try_to_decoding_key(&self) -> Result<jwt::DecodingKey> {
        Ok(match &self.0 {
            jwk::Key::Symmetric { key } => {
                jwt::DecodingKey::from_secret(key.to_vec().as_slice())
            }
            jwk::Key::EC { .. } => {
                // The following will not panic: all EC JWKs have public components due to
                // typing. PEM conversion will always succeed, for the same reason.
                // Hence, jwt::DecodingKey shall have no issue with de-converting.
                jwt::DecodingKey::from_ec_pem(
                    self.0.to_public().unwrap().to_pem().as_bytes(),
                )
                .map_err(errors::any)?
            }
            jwk::Key::RSA { .. } => {
                let pem = self
                    .0
                    .to_public()
                    .unwrap()
                    .try_to_pem()
                    .map_err(errors::any)?;
                jwt::DecodingKey::from_rsa_pem(pem.as_bytes())
                    .map_err(errors::any)?
            }
        })
    }

    pub fn to_decoding_key(&self) -> jwt::DecodingKey {
        self.try_to_decoding_key().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;
    use rsa::traits::{PrivateKeyParts, PublicKeyParts};

    use super::*;

    use crate::services::oidc::jwk_to_public;
    use cim_storage::{
        key::{Keys, VerificationKey},
        Claim, List,
    };
    use mockall::mock;

    mock! {
        pub KeyStore {
            fn list(&self, opts: &(), output: &mut List<Keys>) -> Result<()>;
        }
    }

    #[async_trait]
    impl Interface for MockKeyStore {
        type T = Keys;
        type L = ();
        async fn put(&self, _input: &Self::T) -> Result<()> {
            unimplemented!()
        }
        async fn delete(&self, _input: &Self::T) -> Result<()> {
            unimplemented!()
        }
        async fn get(&self, _output: &mut Self::T) -> Result<()> {
            unimplemented!()
        }
        async fn list(
            &self,
            opts: &Self::L,
            output: &mut List<Self::T>,
        ) -> Result<()> {
            self.list(opts, output)
        }
        async fn count(&self, _opts: &Self::L, _unscoped: bool) -> Result<i64> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn token_encode_decode() {
        let mut key_store = MockKeyStore::new();
        let key_id = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(40)
            .map(char::from)
            .collect::<String>();
        let mut rng = rand::thread_rng();
        let private_key = rsa::RsaPrivateKey::new(&mut rng, 2048).unwrap();
        let mut p = None;
        let mut q = None;
        let primes = private_key.primes();
        match primes.len() {
            1 => {
                p = Some(primes[0].to_bytes_be().into());
            }
            2 => {
                p = Some(primes[0].to_bytes_be().into());
                q = Some(primes[1].to_bytes_be().into());
            }
            _ => {}
        }
        let key = jwk::Key::RSA {
            public: jwk::RsaPublic {
                e: jwk::PublicExponent,
                n: private_key.n().to_bytes_be().into(),
            },
            private: Some(jwk::RsaPrivate {
                d: private_key.d().to_bytes_be().into(),
                p,
                q,
                dp: private_key.dp().map(|v| v.to_bytes_be().into()),
                dq: private_key.dq().map(|v| v.to_bytes_be().into()),
                qi: private_key.qinv().map(|v| v.to_signed_bytes_be().into()),
            }),
        };

        let mut signing_key = jwk::JsonWebKey::new(key);
        signing_key.set_algorithm(jwk::Algorithm::RS256).unwrap();
        signing_key.key_use = Some(jwk::KeyUse::Signing);
        signing_key.key_id = Some(key_id);

        let mut signing_key_pub = signing_key.clone();
        signing_key_pub.key = jwk_to_public(signing_key.key.clone()).unwrap();

        key_store.expect_list().returning(move |_, output| {
            output.data.push(Keys {
                id: 1.to_string(),
                signing_key: signing_key.clone(),
                signing_key_pub: signing_key_pub.clone(),
                verification_keys: vec![VerificationKey {
                    expiry: 12,
                    public_key: signing_key_pub.clone(),
                }],
                next_rotation: 0,
            });
            Ok(())
        });
        let t =
            AccessToken::new(key_store, 30, HashSet::from(["IO".to_owned()]));
        let access_token = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(255)
            .map(char::from)
            .collect::<String>();
        let (token_str, exp) = t
            .token(&Claims {
                claim: Claim {
                    sub: "1".to_owned(),
                    ..Default::default()
                },
                access_token: Some(access_token),
                nonce: "hsjdkjfka".to_owned(),
                aud: "IO".to_owned(),
                ..Default::default()
            })
            .await
            .unwrap();
        println!("{} {}", exp, token_str);
        let claims = t.verify(&token_str).await.unwrap();
        println!("{:#?}", claims);
    }
}
