use std::{collections::HashMap, sync::RwLock};

use async_trait::async_trait;

use slo::{errors, Result};

use crate::{Interface, List, Pagination};

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
    type D = I::D;
    type G = I::G;
    type L = I::L;
    type C = I::C;
    async fn put(&self, input: &mut Self::T, ttl: u64) -> Result<()> {
        self.storage.put(input, ttl).await?;
        // self.cache.remove(&input.id);
        Ok(())
    }

    async fn delete(&self, id: &str, opts: &Self::D) -> Result<()> {
        self.storage.delete(id, opts).await?;
        let mut cache = self.cache.write().map_err(errors::any)?;
        cache.remove(id);
        Ok(())
    }
    async fn get(
        &self,
        id: &str,
        opts: &Self::G,
        output: &mut Self::T,
    ) -> Result<()> {
        // if let Some(v) = self.cache.get(id) {
        //     *output =
        //         serde_json::from_str::<Self::T>(v).map_err(errors::any)?;
        //     return Ok(());
        // };
        self.storage.get(id, opts, output).await?;
        // let value = serde_json::to_string(output).map_err(errors::any)?;
        // self.cache.insert(id.to_string(), value);
        Ok(())
    }
    async fn list(
        &self,
        pagination: &Pagination,
        opts: &Self::L,
        output: &mut List<Self::T>,
    ) -> Result<()> {
        self.storage.list(pagination, opts, output).await?;
        // TODO: cache all
        Ok(())
    }
    async fn count(&self, opts: &Self::C, unscoped: bool) -> Result<i64> {
        self.storage.count(opts, unscoped).await
    }
}
