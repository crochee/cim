use async_trait::async_trait;
use mockall::automock;
use serde::{Deserialize, Serialize};

use slo::Result;

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Keys {
    pub signing_key: KeyValue,
    pub verification_keys: Vec<VerificationKey>,
    pub next_rotation: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VerificationKey {
    pub value: KeyValue,
    pub expiry: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct KeyValue {
    pub id: String,
    pub value: String,
    pub alg: String,
}

#[automock]
#[async_trait]
pub trait KeyStore {
    async fn get_key(&self) -> Result<Keys>;
    async fn update_key(&self, nk: &Keys) -> Result<()>;
    async fn create_key(&self, nk: &Keys) -> Result<()>;
}
