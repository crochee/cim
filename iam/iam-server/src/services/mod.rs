pub mod authorization;
pub mod policies;
pub mod roles;
pub mod usergroups;
pub mod users;

use std::{
    num::NonZeroUsize,
    sync::{Arc, Mutex},
};

use sqlx::MySqlPool;

use crate::{
    config::AppConfig,
    repo::{
        policies::MariadbPolicies, roles::MariadbRoles,
        usergroups::MariadbUserGroups, users::MariadbUsers,
    },
    services::policies::IAMPolicies,
};

use self::{
    authorization::{auth::Auth, matcher::reg::Regexp, DynAuthorizer},
    policies::DynPoliciesService,
    roles::{DynRolesService, IAMRoles},
    usergroups::{DynUserGroupsService, IAMUserGroups},
    users::{DynUsersService, IAMUsers},
};

#[derive(Clone)]
pub struct ServiceRegister {
    pub policies_service: DynPoliciesService,
    pub authorizer: DynAuthorizer,
    pub users_service: DynUsersService,
    pub roles_service: DynRolesService,
    pub user_groups_service: DynUserGroupsService,
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

        let users_repository = Arc::new(MariadbUsers::new(pool.clone()));

        let users_service = Arc::new(IAMUsers::new(users_repository));

        let roles_repository = Arc::new(MariadbRoles::new(pool.clone()));

        let roles_service = Arc::new(IAMRoles::new(roles_repository));

        let user_groups_repository = Arc::new(MariadbUserGroups::new(pool));

        let user_groups_service =
            Arc::new(IAMUserGroups::new(user_groups_repository));

        Ok(Self {
            policies_service,
            authorizer,
            users_service,
            roles_service,
            user_groups_service,
        })
    }
}
