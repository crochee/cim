use std::sync::Arc;

use mockall::automock;
use sqlx::MySqlPool;

pub mod policies;
pub mod pool;
pub mod providers;
pub mod roles;
pub mod usergroups;
pub mod users;
pub mod authreqs;

pub type DynRepository = Arc<dyn Repository + Send + Sync>;

#[automock]
pub trait Repository {
    fn user(&self) -> users::DynUsers;
    fn user_group(&self) -> usergroups::DynUserGroups;
    fn role(&self) -> roles::DynRoles;
    fn policy(&self) -> policies::DynPolicies;
    fn provider(&self) -> providers::DynProviders;
    fn authreq(&self) -> authreqs::DynAuthReqs;
}

pub struct MariadbRepo {
    pool: MySqlPool,
}

impl MariadbRepo {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

impl Repository for MariadbRepo {
    fn user(&self) -> users::DynUsers {
        Arc::new(users::MariadbUsers::new(self.pool.clone()))
    }

    fn user_group(&self) -> usergroups::DynUserGroups {
        Arc::new(usergroups::MariadbUserGroups::new(self.pool.clone()))
    }

    fn role(&self) -> roles::DynRoles {
        Arc::new(roles::MariadbRoles::new(self.pool.clone()))
    }

    fn policy(&self) -> policies::DynPolicies {
        Arc::new(policies::MariadbPolicies::new(self.pool.clone()))
    }

    fn provider(&self) -> providers::DynProviders {
        Arc::new(providers::MariadbProviders::new(self.pool.clone()))
    }
    fn authreq(&self) -> authreqs::DynAuthReqs{
        Arc::new(authreqs::MariadbAuthReqs::new(self.pool.clone()))
    }
}
