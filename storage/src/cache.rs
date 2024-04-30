use std::{collections::HashMap, sync::RwLock};

use async_trait::async_trait;

use slo::{errors, Result};

use crate::{Interface, List, Pagination, Watcher};

pub struct Cacher<I> {
    storage: I,
    cache: RwLock<HashMap<String, String>>,
}

impl<I> Cacher<I> {
    pub fn new(storage: I) -> Self {
        Self {
            storage,
            cache: RwLock::new(HashMap::new()),
        }
    }

    pub fn key(&self, id: &str) -> String {
        format!("{}:{}", crate::type_name!(Self::I), id)
    }
}

#[async_trait]
impl<I: Interface> Interface for Cacher<I> {
    type T = I::T;
    type L = I::L;
    async fn put(
        &self,
        key: &str,
        input: &mut Self::T,
        ttl: u64,
    ) -> Result<()> {
        self.storage.put(key, input, ttl).await?;
        let mut cache = self.cache.write().map_err(errors::any)?;
        cache.remove(key);
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        self.storage.delete(key).await?;
        let mut cache = self.cache.write().map_err(errors::any)?;
        cache.remove(key);
        Ok(())
    }
    async fn get(&self, key: &str, output: &mut Self::T) -> Result<()> {
        {
            let mut cache = self.cache.read().map_err(errors::any)?;
            if let Some(v) = cache.get(key) {
                *output =
                    serde_json::from_str::<Self::T>(v).map_err(errors::any)?;
                return Ok(());
            }
        }

        let mut cache = self.cache.write().map_err(errors::any)?;
        self.storage.get(key, output).await?;
        let value = serde_json::to_string(output).map_err(errors::any)?;
        cache.insert(key.to_string(), value);
        Ok(())
    }
    async fn list(
        &self,
        key: &str,
        pagination: &Pagination,
        opts: &Self::L,
        output: &mut List<Self::T>,
    ) -> Result<()> {
        self.storage.list(key, pagination, opts, output).await?;
        // TODO: cache all
        Ok(())
    }
    async fn watch<W>(&self, key: &str, opts: &Self::L) -> Result<W>
    where
        W: Watcher<T = Self::T>,
    {
        self.storage.watch(key, opts).await
    }
    async fn count(&self, opts: &Self::L, unscoped: bool) -> Result<i64> {
        self.storage.count(opts, unscoped).await
    }
}
