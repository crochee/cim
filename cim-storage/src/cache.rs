use std::{collections::HashMap, sync::RwLock};

use async_trait::async_trait;

use cim_slo::{errors, type_name, Result};
use cim_watch::{WatchGuard, Watcher};

use crate::{Event, Interface, List};

pub struct Cacher<I> {
    storage: I,
    cache: RwLock<HashMap<String, String>>,
    prefix: String,
}

impl<I> Cacher<I> {
    pub fn new(storage: I, prefix: String) -> Self {
        Self {
            storage,
            cache: RwLock::new(HashMap::new()),
            prefix,
        }
    }

    pub fn key(&self, uid: Option<&str>) -> String {
        let type_name_str = type_name!(Self::I::T);
        let mut buf = String::new();
        buf.push_str(&self.prefix);
        buf.push('/');
        buf.push_str(type_name_str);
        if let Some(uid) = uid {
            buf.push('/');
            buf.push_str(uid);
        }
        buf
    }
}

#[async_trait]
impl<I> Interface for Cacher<I>
where
    I: Interface,
{
    type T = I::T;
    type L = I::L;

    async fn put(&self, input: &Self::T, ttl: u64) -> Result<()> {
        self.storage.put(input, ttl).await?;
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<()> {
        self.storage.delete(id).await?;
        let mut cache = self.cache.write().map_err(errors::any)?;
        let key = self.key(Some(id));
        cache.remove(&key);
        Ok(())
    }
    async fn get(&self, id: &str, output: &mut Self::T) -> Result<()> {
        let key = self.key(Some(id));
        {
            let cache = self.cache.read().map_err(errors::any)?;
            if let Some(v) = cache.get(&key) {
                *output =
                    serde_json::from_str::<Self::T>(v).map_err(errors::any)?;
                return Ok(());
            }
        }

        self.storage.get(id, output).await?;
        let mut cache = self.cache.write().map_err(errors::any)?;
        let value = serde_json::to_string(output).map_err(errors::any)?;
        cache.insert(key, value);
        Ok(())
    }
    async fn list(
        &self,
        opts: &Self::L,
        output: &mut List<Self::T>,
    ) -> Result<()> {
        self.storage.list(opts, output).await?;
        // TODO: cache all
        Ok(())
    }
    fn watch<W: Watcher<Event<Self::T>>>(
        &self,
        handler: W,
    ) -> Box<dyn WatchGuard + Send> {
        self.storage.watch(handler)
    }
    async fn count(&self, opts: &Self::L, unscoped: bool) -> Result<i64> {
        self.storage.count(opts, unscoped).await
    }
}
