use std::num::NonZeroUsize;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use async_trait::async_trait;
use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use sqlx::MySqlPool;
use tracing::info;

use crate::{
    errors::WithBacktrace,
    services::{
        authentication::key::{KeyRotator, RotationStrategy},
        authorization::matcher::reg::Regexp,
    },
    store::MariadbStore,
    AppConfig,
};

pub struct App {
    pub config: AppConfig,
    pub store: MariadbStore,
    pub matcher: Regexp,
    pub key_rotator: KeyRotator<MariadbStore>,
}

#[derive(Clone)]
pub struct AppState(pub Arc<App>);

// deref so you can still access the inner fields easily
impl Deref for AppState {
    type Target = App;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AppState
where
    Self: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = WithBacktrace;
    async fn from_request_parts(
        _: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self::from_ref(state))
    }
}

impl App {
    pub fn new(pool: MySqlPool, config: AppConfig) -> Result<Self> {
        info!("initializing utility services...");

        let matcher = Regexp {
            lru: Mutex::new(lru::LruCache::new(
                NonZeroUsize::new(config.cache_size).ok_or_else(|| {
                    anyhow::anyhow!("panic on {}", config.cache_size)
                })?,
            )),
        };
        let store = MariadbStore::new(pool);

        let key_rotator = KeyRotator::new(
            store.clone(),
            RotationStrategy {
                rotation_frequency: 6 * 60 * 60,
                keep: 6 * 60 * 60,
            },
        );

        info!("feature services successfully initialized!");
        Ok(Self {
            config,
            store,
            matcher,
            key_rotator,
        })
    }
}
