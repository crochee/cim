pub mod authorization;
pub mod policies;
pub mod users;

use std::{
    num::NonZeroUsize,
    sync::{Arc, Mutex},
};

use sqlx::MySqlPool;

use crate::{
    config::AppConfig,
    repo::{policies::MariadbPolicies, users::MariadbUsers},
    services::policies::IAMPolicies,
};

use self::{
    authorization::{auth::Auth, matcher::reg::Regexp, DynAuthorizer},
    policies::DynPoliciesService,
    users::{DynUsersService, IAMUsers},
};

#[derive(Clone)]
pub struct ServiceRegister {
    pub policies_service: DynPoliciesService,
    pub authorizer: DynAuthorizer,
    pub users_service: DynUsersService,
}

impl ServiceRegister {
    pub fn new(
        pool: MySqlPool,
        config: Arc<AppConfig>,
    ) -> anyhow::Result<Self> {
        let policies_repository = Arc::new(MariadbPolicies::new(pool.clone()));

        let policies_service =
            Arc::new(IAMPolicies::new(policies_repository.clone()));

        let authorizer = Arc::new(Auth::new(
            policies_repository,
            Regexp {
                lru: Arc::new(Mutex::new(lru::LruCache::new(
                    NonZeroUsize::new(config.cache_size).ok_or_else(|| {
                        anyhow::anyhow!("panic on {}", config.cache_size)
                    })?,
                ))),
            },
        ));

        let users_repository = Arc::new(MariadbUsers::new(pool));

        let users_service = Arc::new(IAMUsers::new(users_repository));

        Ok(Self {
            policies_service,
            authorizer,
            users_service,
        })
    }
}
