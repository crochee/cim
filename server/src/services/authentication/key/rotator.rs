use async_trait::async_trait;
use chrono::Utc;
use rand::Rng;

use cim_core::{next_id, Code, Result};

use crate::{
    models::key::{KeyValue, VerificationKey},
    store::Store,
};

use super::{Keys, KeysStore};

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

#[async_trait]
impl<S> KeysStore for KeyRotator<S>
where
    S: Store,
{
    async fn get(&self) -> Result<Keys> {
        self.store.get_key().await
    }
}

impl<S> KeyRotator<S>
where
    S: Store,
{
    pub async fn rotate(&self) -> Result<()> {
        let keys = match self.store.get_key().await {
            Ok(v) => v,
            Err(err) => {
                if !err.eq(&Code::not_found("")) {
                    return Err(err);
                }

                let nk = self.get_key(&Keys::default())?;
                return self.store.create_key(&nk).await;
            }
        };
        if keys.next_rotation > Self::time_now() {
            return Ok(());
        }
        let nk = self.get_key(&keys)?;
        self.store.update_key(&nk).await
    }

    fn time_now() -> i64 {
        Utc::now().timestamp()
    }

    fn key_generator() -> String {
        rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(65)
            .map(char::from)
            .collect::<String>()
    }

    fn get_key(&self, nk: &Keys) -> Result<Keys> {
        let id = next_id().map_err(Code::any)?.to_string();
        let key = KeyValue {
            id,
            value: Self::key_generator(),
            alg: String::from("HS256"),
        };
        let now_time = Self::time_now();
        if nk.next_rotation > now_time {
            return Err(Code::Any(anyhow::anyhow!(
                "keys already rotated by another server instance"
            ))
            .with());
        }
        let mut result = Keys {
            signing_key: key.clone(),
            ..Default::default()
        };
        result.verification_keys.push(VerificationKey {
            value: key,
            expiry: now_time + self.strategy.keep,
        });
        for vk in nk.verification_keys.iter() {
            if vk.expiry > now_time {
                result.verification_keys.push(vk.clone());
            }
        }
        result.next_rotation =
            Self::time_now() + self.strategy.rotation_frequency;
        Ok(result)
    }
}

#[cfg(test)]
mod test {

    use chrono::Utc;
    use cim_core::next_id;

    use crate::{
        models::key::{KeyValue, Keys, VerificationKey},
        services::authentication::key::{KeyRotator, RotationStrategy},
        store::MockStore,
    };

    #[tokio::test]
    async fn rotate() {
        let mut store = MockStore::new();

        store.expect_get_key().times(..).return_once(|| {
            let id = next_id().unwrap().to_string();
            let key = KeyValue {
                id,
                value: "sdjh".to_string(),
                alg: String::from("RS256"),
            };

            Ok(Keys {
                signing_key: key.clone(),
                verification_keys: vec![VerificationKey {
                    value: key,
                    expiry: 100,
                }],
                next_rotation: Utc::now().timestamp() + 60,
            })
        });

        let strategy = RotationStrategy {
            rotation_frequency: 6 * 60 * 60,
            keep: 6 * 60 * 60,
        };
        let key_store = KeyRotator::new(store, strategy);

        key_store.rotate().await.unwrap();
    }
}
