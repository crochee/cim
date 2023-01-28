pub mod authorization;
pub mod policies;
pub mod rolebindings;
pub mod roles;
pub mod usergroupbindings;
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
        policies::MariadbPolicies, rolebindings::MariadbRoleBindings,
        roles::MariadbRoles, usergroupbindings::MariadbUserGroupBindings,
        usergroups::MariadbUserGroups, users::MariadbUsers,
    },
    services::policies::IAMPolicies,
};

use self::{
    authorization::{auth::Auth, matcher::reg::Regexp, DynAuthorizer},
    policies::DynPoliciesService,
    rolebindings::{DynRoleBindingsService, IAMRoleBindings},
    roles::{DynRolesService, IAMRoles},
    usergroupbindings::{DynUserGroupBindingsService, IAMUserGroupBindings},
    usergroups::{DynUserGroupsService, IAMUserGroups},
    users::{DynUsersService, IAMUsers},
};

#[derive(Clone)]
pub struct ServiceRegister {
    pub policies_service: DynPoliciesService,
    pub authorizer: DynAuthorizer,
    pub users_service: DynUsersService,
    pub roles_service: DynRolesService,
    pub usergroups_service: DynUserGroupsService,
    pub rolebindings_service: DynRoleBindingsService,
    pub usergroupbindings_service: DynUserGroupBindingsService,
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

        let user_groups_repository =
            Arc::new(MariadbUserGroups::new(pool.clone()));

        let usergroups_service =
            Arc::new(IAMUserGroups::new(user_groups_repository));

        let rolebindings_repository =
            Arc::new(MariadbRoleBindings::new(pool.clone()));

        let rolebindings_service =
            Arc::new(IAMRoleBindings::new(rolebindings_repository));

        let usergroupbindings_repository =
            Arc::new(MariadbUserGroupBindings::new(pool));

        let usergroupbindings_service =
            Arc::new(IAMUserGroupBindings::new(usergroupbindings_repository));

        Ok(Self {
            policies_service,
            authorizer,
            users_service,
            roles_service,
            usergroups_service,
            rolebindings_service,
            usergroupbindings_service,
        })
    }
}
