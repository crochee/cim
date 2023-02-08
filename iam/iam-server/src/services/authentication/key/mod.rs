mod rotator;

use async_trait::async_trait;
use mockall::automock;

use cim_core::Result;

pub use rotator::{KeyRotator, RotationStrategy};

use crate::models::key::Keys;

#[automock]
#[async_trait]
pub trait KeysStore: Send + Sync {
    async fn get(&self) -> Result<Keys>;
}
