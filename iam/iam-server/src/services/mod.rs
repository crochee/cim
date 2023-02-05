pub mod authentication;
pub mod authorization;
pub mod policies;
pub mod roles;
pub mod templates;
pub mod usergroups;
pub mod users;

use std::{
    num::NonZeroUsize,
    sync::{Arc, Mutex},
};

use mockall::automock;
use sqlx::MySqlPool;

use crate::{
    config::AppConfig,
    repo::{DynRepository, MariadbRepo},
};

use self::authorization::{matcher::reg::Regexp, DynMatcher, IAMAuth};

pub type DynService = Arc<dyn Service + Send + Sync>;

#[automock]
pub trait Service {
    fn user(&self) -> users::DynUsers;
    fn user_group(&self) -> usergroups::DynUserGroups;
    fn role(&self) -> roles::DynRoles;
    fn policy(&self) -> policies::DynPolicies;
    fn authorization(&self) -> authorization::DynAuthorizer;
    fn authentication(&self) -> authentication::DynAuthenticator;
}

#[derive(Clone)]
pub struct ServiceRegister {
    pub repo: DynRepository,
    pub matcher: DynMatcher,
}

impl Service for ServiceRegister {
    fn user(&self) -> users::DynUsers {
        Arc::new(users::IAMUsers::new(self.repo.clone()))
    }

    fn user_group(&self) -> usergroups::DynUserGroups {
        Arc::new(usergroups::IAMUserGroups::new(self.repo.clone()))
    }

    fn role(&self) -> roles::DynRoles {
        Arc::new(roles::IAMRoles::new(self.repo.clone()))
    }

    fn policy(&self) -> policies::DynPolicies {
        Arc::new(policies::IAMPolicies::new(self.repo.clone()))
    }

    fn authorization(&self) -> authorization::DynAuthorizer {
        Arc::new(IAMAuth::new(self.repo.clone(), self.matcher.clone()))
    }

    fn authentication(&self) -> authentication::DynAuthenticator {
        Arc::new(authentication::IAMAuthenticator::new(self.repo.clone()))
    }
}

impl ServiceRegister {
    pub fn new(
        pool: MySqlPool,
        config: Arc<AppConfig>,
    ) -> anyhow::Result<Self> {
        let repo = Arc::new(MariadbRepo::new(pool));
        let matcher = Arc::new(Regexp {
            lru: Mutex::new(lru::LruCache::new(
                NonZeroUsize::new(config.cache_size).ok_or_else(|| {
                    anyhow::anyhow!("panic on {}", config.cache_size)
                })?,
            )),
        });

        Ok(Self { repo, matcher })
    }
}
