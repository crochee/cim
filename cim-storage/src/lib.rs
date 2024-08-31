pub mod authcode;
pub mod authrequest;
// mod cache;
pub mod client;
pub mod connector;
pub mod convert;
pub mod group;
pub mod group_user;
pub mod key;
mod mariadb;
mod model;
pub mod offlinesession;
pub mod policy;
pub mod policy_binding;
mod pool;
pub mod refresh_token;
pub mod role;
pub mod role_binding;
pub mod user;
mod watch;

use async_trait::async_trait;
use cim_watch::{WatchGuard, Watcher};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use cim_slo::Result;

pub use mariadb::*;
pub use model::{Claim, ClaimOpts, List, Pagination, ID};
pub use pool::connection_manager;
pub use watch::WatchStore;

#[async_trait]
pub trait Interface: Sync {
    type T: DeserializeOwned
        + Serialize
        + Send
        + Sync
        + PartialEq
        + Clone
        + Default
        + 'static;

    type L: Sync;

    async fn put(&self, input: &Self::T, ttl: u64) -> Result<()>;
    async fn delete(&self, input: &Self::T) -> Result<()>;
    async fn get(&self, output: &mut Self::T) -> Result<()>;
    async fn list(
        &self,
        opts: &Self::L,
        output: &mut List<Self::T>,
    ) -> Result<()>;
    async fn count(&self, opts: &Self::L, unscoped: bool) -> Result<i64>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event<T> {
    Add(T),
    Put(T),
    Delete(T),
}

impl<T> Event<T> {
    pub fn get(&self) -> &T {
        match self {
            Event::Add(t) => t,
            Event::Put(t) => t,
            Event::Delete(t) => t,
        }
    }
}

#[async_trait]
pub trait WatchInterface: Interface {
    fn watch<W: Watcher<Event<Self::T>>>(
        &self,
        since_modify: usize,
        handler: W,
    ) -> Box<dyn WatchGuard + Send>;

    async fn create(&self, input: &Self::T) -> Result<()>;
}
