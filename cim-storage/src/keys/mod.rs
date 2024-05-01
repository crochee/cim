mod mariadb;

use async_trait::async_trait;
use jsonwebkey as jwk;
use mockall::automock;
use serde::{Deserialize, Serialize};

pub use mariadb::KeyImpl;

use cim_slo::Result;

#[derive(Debug, Deserialize, Serialize)]
pub struct Keys {
    pub signing_key: jwk::JsonWebKey,
    pub signing_key_pub: jwk::JsonWebKey,
    pub verification_keys: Vec<VerificationKey>,
    pub next_rotation: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VerificationKey {
    pub public_key: jwk::JsonWebKey,
    pub expiry: i64,
}

#[automock]
#[async_trait]
pub trait KeyStore {
    async fn get_key(&self) -> Result<Keys>;
    async fn update_key(&self, nk: &Keys) -> Result<()>;
    async fn create_key(&self, nk: &Keys) -> Result<()>;
}
