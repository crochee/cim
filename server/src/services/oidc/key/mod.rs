use chrono::Utc;
use jsonwebkey as jwk;
use rand::Rng;
use rsa::traits::{PrivateKeyParts, PublicKeyParts};
use serde::Serialize;
use slo::{errors, Result};
use utoipa::ToSchema;

use storage::keys::*;

use super::jwk_to_public;

#[derive(Debug, Serialize, ToSchema)]
pub struct JsonWebKeySet {
    pub keys: Vec<jwk::JsonWebKey>,
}

impl JsonWebKeySet {
    #[allow(dead_code)]
    pub fn keys(&self, kid: &str) -> Vec<jwk::JsonWebKey> {
        self.keys
            .iter()
            .filter(|k| k.key_id == Some(kid.to_string()))
            .cloned()
            .collect()
    }
}

#[derive(Clone)]
pub struct KeyRotator<S> {
    store: S,
    strategy: RotationStrategy,
}

#[derive(Clone)]
pub struct RotationStrategy {
    pub rotation_frequency: i64,
    pub keep: i64,
}

impl<S> KeyRotator<S> {
    pub fn new(store: S, strategy: RotationStrategy) -> Self {
        Self { store, strategy }
    }
}

impl<S> KeyRotator<S>
where
    S: KeyStore,
{
    pub async fn rotate(&self) -> Result<()> {
        let mut keys = match self.store.get_key().await {
            Ok(v) => v,
            Err(err) => {
                if !err.eq(&errors::not_found("")) {
                    return Err(err);
                }
                let (signing_key, signing_key_pub) = self.create_key()?;
                let now_time = Self::time_now();
                return self
                    .store
                    .create_key(&Keys {
                        signing_key,
                        signing_key_pub: signing_key_pub.clone(),
                        verification_keys: vec![VerificationKey {
                            expiry: now_time + self.strategy.keep,
                            public_key: signing_key_pub,
                        }],
                        next_rotation: now_time
                            + self.strategy.rotation_frequency,
                    })
                    .await;
            }
        };
        if keys.next_rotation > Self::time_now() {
            return Ok(());
        }
        self.update_key(&mut keys)?;
        self.store.update_key(&keys).await
    }

    fn time_now() -> i64 {
        Utc::now().timestamp()
    }

    fn key_generator(take: usize) -> String {
        rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(take)
            .map(char::from)
            .collect::<String>()
    }

    fn create_key(&self) -> Result<(jwk::JsonWebKey, jwk::JsonWebKey)> {
        let mut rng = rand::thread_rng();
        let private_key =
            rsa::RsaPrivateKey::new(&mut rng, 2048).map_err(errors::any)?;

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
        signing_key
            .set_algorithm(jwk::Algorithm::RS256)
            .map_err(errors::any)?;
        signing_key.key_use = Some(jwk::KeyUse::Signing);
        signing_key.key_id = Some(Self::key_generator(40));

        let mut signing_key_pub = signing_key.clone();

        signing_key_pub.key =
            jwk_to_public(signing_key.key.clone()).map_err(errors::any)?;
        Ok((signing_key, signing_key_pub))
    }

    fn update_key(&self, nk: &mut Keys) -> Result<()> {
        let now_time = Self::time_now();
        if nk.next_rotation > now_time {
            return Err(errors::anyhow(anyhow::anyhow!(
                "keys already rotated by another server instance"
            )));
        }
        let (signing_key, signing_key_pub) = self.create_key()?;

        // 删除过期的key
        nk.verification_keys.retain(|vk| vk.expiry > now_time);

        nk.verification_keys.push(VerificationKey {
            expiry: now_time + self.strategy.keep,
            public_key: signing_key_pub.clone(),
        });
        nk.signing_key = signing_key;
        nk.signing_key_pub = signing_key_pub;
        nk.next_rotation = now_time + self.strategy.rotation_frequency;
        Ok(())
    }
}
