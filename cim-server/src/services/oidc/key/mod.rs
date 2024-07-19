use chrono::Utc;
use jsonwebkey as jwk;
use rand::Rng;
use rsa::traits::{PrivateKeyParts, PublicKeyParts};
use serde::Serialize;
use utoipa::ToSchema;

use cim_slo::{errors, next_id, Result};
use cim_storage::{
    key::{Keys, VerificationKey},
    Interface, List,
};

use super::jwk_to_public;

#[derive(Debug, Serialize, ToSchema)]
pub struct JsonWebKeySet {
    pub keys: Vec<jwk::JsonWebKey>,
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
    S: Interface<T = Keys, L = ()>,
{
    pub async fn rotate(&self) -> Result<()> {
        let mut output: List<Keys> = List::default();
        self.store.list(&(), &mut output).await?;

        let mut keys = Keys::default();

        if !output.data.is_empty() {
            keys = output.data.remove(0);
        } else {
            let (signing_key, signing_key_pub) = self.create_key()?;
            let now_time = Self::time_now();
            let id = next_id().map_err(errors::any)?;
            keys.id = id.to_string();
            keys.signing_key = signing_key;
            keys.signing_key_pub = signing_key_pub.clone();
            keys.verification_keys = vec![VerificationKey {
                expiry: now_time + self.strategy.keep,
                public_key: signing_key_pub,
            }];
            keys.next_rotation = now_time + self.strategy.rotation_frequency;
        }
        if keys.next_rotation > Self::time_now() {
            return Ok(());
        }
        self.update_key(&mut keys)?;
        self.store.put(&keys, 0).await
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

        let mut new_keys = vec![VerificationKey {
            expiry: now_time + self.strategy.keep,
            public_key: signing_key_pub.clone(),
        }];
        new_keys.append(&mut nk.verification_keys);
        nk.verification_keys = new_keys;

        nk.signing_key = signing_key;
        nk.signing_key_pub = signing_key_pub;
        nk.next_rotation = now_time + self.strategy.rotation_frequency;
        Ok(())
    }
}
