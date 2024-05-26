pub mod authcode;
pub mod authrequest;
mod cache;
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

use cim_slo::Result;
use cim_watch::Watcher;

#[derive(Clone, Debug)]
pub enum Event<T> {
    Put(T),
    Delete(T),
}

#[async_trait]
pub trait Interface: Sync {
    type T: DeserializeOwned + Serialize + Send + Sync + PartialEq;
    type L: Sync;

    async fn put(&self, id: &str, input: &Self::T, ttl: u64) -> Result<()>;
    async fn delete(&self, id: &str) -> Result<()>;
    async fn get(&self, id: &str, output: &mut Self::T) -> Result<()>;
    async fn list(
        &self,
        pagination: &Pagination,
        opts: &Self::L,
        output: &mut List<Self::T>,
    ) -> Result<()>;
    fn watch<W: Watcher<Event<Self::T>>>(
        &self,
        handler: W,
        remove: impl Fn() + Send + 'static,
    ) -> Box<dyn Fn() + Send>;

    async fn count(&self, opts: &Self::L, unscoped: bool) -> Result<i64>;
}
