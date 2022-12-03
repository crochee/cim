mod mariadb;

use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;

pub type DynPoliciesRepository = Arc<dyn PoliciesRepository + Send + Sync>;
pub use mariadb::MariadbPolicies;

#[automock]
#[async_trait]
pub trait PoliciesRepository {}
