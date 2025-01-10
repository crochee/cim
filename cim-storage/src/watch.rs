use async_trait::async_trait;
use chrono::Utc;

use cim_slo::Result;
use cim_watch::{WatchGuard, Watcher, WatcherHub};

use crate::{Event, Interface, List, WatchInterface};

#[derive(Clone)]
pub struct WatchStore<I: Interface> {
    store: I,
    watch_hub: WatcherHub<Event<I::T>>,
}

impl<I: Interface> WatchStore<I> {
    pub fn new(store: I) -> Self {
        Self {
            store,
            watch_hub: WatcherHub::default(),
        }
    }
}

#[async_trait]
impl<I: Interface> WatchInterface for WatchStore<I> {
    fn watch<W: Watcher<Event<Self::T>>>(
        &self,
        since_modify: usize,
        handler: W,
    ) -> Box<dyn WatchGuard + Send> {
        self.watch_hub.watch(since_modify, handler)
    }

    async fn create(&self, input: &Self::T) -> Result<()> {
        self.store.put(input).await?;
        self.watch_hub
            .notify(Utc::now().timestamp() as usize, Event::Add(input.clone()));
        Ok(())
    }
}

#[async_trait]
impl<I: Interface> Interface for WatchStore<I> {
    type T = I::T;
    type L = I::L;
    async fn put(&self, input: &I::T) -> Result<()> {
        self.store.put(input).await?;
        self.watch_hub
            .notify(Utc::now().timestamp() as usize, Event::Put(input.clone()));
        Ok(())
    }

    async fn delete(&self, input: &I::T) -> Result<()> {
        self.store.delete(input).await?;
        self.watch_hub.notify(
            Utc::now().timestamp() as usize,
            Event::Delete(input.clone()),
        );
        Ok(())
    }

    async fn get(&self, output: &mut I::T) -> Result<()> {
        self.store.get(output).await
    }

    async fn list(
        &self,
        opts: &Self::L,
        output: &mut List<Self::T>,
    ) -> Result<()> {
        self.store.list(opts, output).await
    }

    async fn count(&self, opts: &Self::L, unscoped: bool) -> Result<i64> {
        self.store.count(opts, unscoped).await
    }
}
