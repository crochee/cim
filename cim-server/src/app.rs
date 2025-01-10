use std::{collections::HashSet, ops::Deref, sync::Arc};

use anyhow::Result;
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
    pub key_rotator: KeyRotator<cim_storage::KeysImpl>,
    pub access_token: AccessToken<cim_storage::KeysImpl>,
}

impl App {
    pub fn new(pool: MySqlPool, config: AppConfig) -> Result<Self> {
        info!("initializing utility services...");

        let matcher = Pim::new(Regexp::new(config.cache_size)?);

        let store = Store::new(pool.clone());

        let key_rotator = KeyRotator::new(
            cim_storage::KeysImpl::new(pool.clone()),
            RotationStrategy {
                rotation_frequency: 60 * 60,
                keep: 60 * 60,
            },
        );

        let access_token = AccessToken::new(
            cim_storage::KeysImpl::new(pool),
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
    pub user: cim_storage::WatchStore<cim_storage::UserImpl>,
    pub role: cim_storage::WatchStore<cim_storage::RoleImpl>,
    pub role_binding: cim_storage::WatchStore<cim_storage::RoleBindingImpl>,
    pub group: cim_storage::WatchStore<cim_storage::GroupImpl>,
    pub group_user: cim_storage::WatchStore<cim_storage::GroupUserImpl>,
    pub statement: cim_storage::PolicyImpl,
    pub policy: cim_storage::WatchStore<cim_storage::PolicyImpl>,
    pub policy_binding: cim_storage::WatchStore<cim_storage::PolicyBindingImpl>,

    pub key: cim_storage::KeysImpl,
    pub auth_request: cim_storage::AuthRequestImpl,
    pub auth_code: cim_storage::AuthCodeImpl,
    pub connector: cim_storage::ConnectorImpl,
    pub client: cim_storage::ClientImpl,
    pub refresh: cim_storage::RefreshTokenImpl,
    pub offline_session: cim_storage::OfflineSessionImpl,
}

impl Store {
    pub fn new(pool: MySqlPool) -> Self {
        let user = cim_storage::WatchStore::new(cim_storage::UserImpl::new(
            pool.clone(),
        ));
        let role = cim_storage::WatchStore::new(cim_storage::RoleImpl::new(
            pool.clone(),
        ));
        let role_binding = cim_storage::WatchStore::new(
            cim_storage::RoleBindingImpl::new(pool.clone()),
        );
        let group = cim_storage::WatchStore::new(cim_storage::GroupImpl::new(
            pool.clone(),
        ));
        let group_user = cim_storage::WatchStore::new(
            cim_storage::GroupUserImpl::new(pool.clone()),
        );
        let statement = cim_storage::PolicyImpl::new(pool.clone());
        let policy = cim_storage::WatchStore::new(statement.clone());

        let policy_binding = cim_storage::WatchStore::new(
            cim_storage::PolicyBindingImpl::new(pool.clone()),
        );
        let key = cim_storage::KeysImpl::new(pool.clone());
        let auth_request = cim_storage::AuthRequestImpl::new(pool.clone());
        let auth_code = cim_storage::AuthCodeImpl::new(pool.clone());
        let connector = cim_storage::ConnectorImpl::new(pool.clone());
        let client = cim_storage::ClientImpl::new(pool.clone());
        let refresh = cim_storage::RefreshTokenImpl::new(pool.clone());
        let offline_session = cim_storage::OfflineSessionImpl::new(pool);
        Self {
            user,
            role,
            role_binding,
            group,
            group_user,
            statement,
            policy,
            policy_binding,
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
