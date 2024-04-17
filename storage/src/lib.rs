pub mod authcode;
pub mod authrequest;
pub mod client;
pub mod connector;
pub mod convert;
pub mod groups;
pub mod keys;
mod model;
pub mod offlinesession;
pub mod policies;
mod pool;
pub mod refresh;
pub mod roles;
pub mod users;

pub use model::{Claim, ClaimOpts, List, Pagination, ID};
pub use pool::connection_manager;

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use slo::Result;

#[async_trait]
pub trait Interface {
    type T: DeserializeOwned + Serialize + Sync;
    type D: Sync;
    type G: Sync;
    type L: Sync;
    type C: Sync;
    async fn put(&self, input: &mut Self::T, ttl: u64) -> Result<()>;
    async fn delete(&self, id: &str, opts: &Self::D) -> Result<()>;
    async fn get(
        &self,
        id: &str,
        opts: &Self::G,
        output: &mut Self::T,
    ) -> Result<()>;
    async fn list(
        &self,
        pagination: &Pagination,
        opts: &Self::L,
        output: &mut List<Self::T>,
    ) -> Result<()>;
    async fn count(&self, opts: &Self::C, unscoped: bool) -> Result<i64>;
}
