use std::{collections::HashSet, ops::Deref, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;
use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use pim::{Pim, Regexp};
use sqlx::MySqlPool;
use tracing::info;

use crate::{
    services::oidc::{
        key::{KeyRotator, RotationStrategy},
        token::AccessToken,
    },
    AppConfig,
};

pub struct App {
    pub config: AppConfig,
    pub matcher: Pim<Regexp>,
    pub store: Store,
    pub key_rotator: KeyRotator<storage::keys::mariadb::KeyImpl>,
    pub access_token: AccessToken<storage::keys::mariadb::KeyImpl>,
}

impl App {
    pub fn new(pool: MySqlPool, config: AppConfig) -> Result<Self> {
        info!("initializing utility services...");

        let matcher = Pim::new(Regexp::new(config.cache_size)?);

        let store = Store::new(pool.clone());

        let key_rotator = KeyRotator::new(
            storage::keys::mariadb::KeyImpl::new(pool.clone()),
            RotationStrategy {
                rotation_frequency: 6 * 60 * 60,
                keep: 24 * 60 * 60,
            },
        );

        let access_token = AccessToken::new(
            storage::keys::mariadb::KeyImpl::new(pool),
            config.expiration,
            HashSet::new(),
            config.endpoint.clone(),
        );
        info!("feature services successfully initialized!");
        Ok(Self {
            config,
            store,
            matcher,
            key_rotator,
            access_token,
        })
    }
}

pub struct Store {
    pub user: storage::users::mariadb::UserImpl,
    pub role: storage::roles::mariadb::RoleImpl,
    pub group: storage::groups::mariadb::GroupImpl,
    pub policy: storage::policies::mariadb::PolicyImpl,
    pub key: storage::keys::mariadb::KeyImpl,
    pub auth_request: storage::authrequest::AuthRequestImpl,
    pub auth_code: storage::authcode::AuthCodeImpl,
    pub connector: storage::connector::ConnectorImpl,
    pub client: storage::client::ClientImpl,
    pub refresh: storage::refresh::RefreshTokenImpl,
    pub offline_session: storage::offlinesession::OfflineSessionImpl,
}

impl Store {
    pub fn new(pool: MySqlPool) -> Self {
        let user = storage::users::mariadb::UserImpl::new(pool.clone());
        let role = storage::roles::mariadb::RoleImpl::new(pool.clone());
        let group = storage::groups::mariadb::GroupImpl::new(pool.clone());
        let policy = storage::policies::mariadb::PolicyImpl::new(pool.clone());
        let key = storage::keys::mariadb::KeyImpl::new(pool.clone());
        let auth_request =
            storage::authrequest::AuthRequestImpl::new(pool.clone());
        let auth_code = storage::authcode::AuthCodeImpl::new(pool.clone());
        let connector = storage::connector::ConnectorImpl::new(pool.clone());
        let client = storage::client::ClientImpl::new(pool.clone());
        let refresh = storage::refresh::RefreshTokenImpl::new(pool.clone());
        let offline_session =
            storage::offlinesession::OfflineSessionImpl::new(pool);
        Self {
            user,
            role,
            group,
            policy,
            key,
            auth_request,
            auth_code,
            connector,
            client,
            refresh,
            offline_session,
        }
    }
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
    type Rejection = slo::errors::WithBacktrace;
    async fn from_request_parts(
        _: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self::from_ref(state))
    }
}
