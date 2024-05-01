use std::{collections::HashSet, ops::Deref, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;
use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use sqlx::MySqlPool;
use tracing::info;

use cim_pim::{Pim, Regexp};
use cim_slo::errors;

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
    pub key_rotator: KeyRotator<cim_storage::keys::KeyImpl>,
    pub access_token: AccessToken<cim_storage::keys::KeyImpl>,
}

impl App {
    pub fn new(pool: MySqlPool, config: AppConfig) -> Result<Self> {
        info!("initializing utility services...");

        let matcher = Pim::new(Regexp::new(config.cache_size)?);

        let store = Store::new(pool.clone());

        let key_rotator = KeyRotator::new(
            cim_storage::keys::KeyImpl::new(pool.clone()),
            RotationStrategy {
                rotation_frequency: 6 * 60 * 60,
                keep: 24 * 60 * 60,
            },
        );

        let access_token = AccessToken::new(
            cim_storage::keys::KeyImpl::new(pool),
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
    pub user: cim_storage::users::UserImpl,
    pub role: cim_storage::roles::RoleImpl,
    pub group: cim_storage::groups::GroupImpl,
    pub policy: cim_storage::policies::PolicyImpl,
    pub key: cim_storage::keys::KeyImpl,
    pub auth_request: cim_storage::authrequest::AuthRequestImpl,
    pub auth_code: cim_storage::authcode::AuthCodeImpl,
    pub connector: cim_storage::connector::ConnectorImpl,
    pub client: cim_storage::client::ClientImpl,
    pub refresh: cim_storage::refresh::RefreshTokenImpl,
    pub offline_session: cim_storage::offlinesession::OfflineSessionImpl,
}

impl Store {
    pub fn new(pool: MySqlPool) -> Self {
        let user = cim_storage::users::UserImpl::new(pool.clone());
        let role = cim_storage::roles::RoleImpl::new(pool.clone());
        let group = cim_storage::groups::GroupImpl::new(pool.clone());
        let policy = cim_storage::policies::PolicyImpl::new(pool.clone());
        let key = cim_storage::keys::KeyImpl::new(pool.clone());
        let auth_request =
            cim_storage::authrequest::AuthRequestImpl::new(pool.clone());
        let auth_code = cim_storage::authcode::AuthCodeImpl::new(pool.clone());
        let connector =
            cim_storage::connector::ConnectorImpl::new(pool.clone());
        let client = cim_storage::client::ClientImpl::new(pool.clone());
        let refresh = cim_storage::refresh::RefreshTokenImpl::new(pool.clone());
        let offline_session =
            cim_storage::offlinesession::OfflineSessionImpl::new(pool);
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
    type Rejection = errors::WithBacktrace;
    async fn from_request_parts(
        _: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self::from_ref(state))
    }
}
