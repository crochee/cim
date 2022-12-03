pub mod policies;
pub mod authz;
pub mod req;

use std::sync::Arc;

use regex::Regex;
use serde::Deserialize;
use sqlx::MySqlPool;

use policies::DynPoliciesService;
use validator::{Validate, ValidationError};

use crate::{
    config::AppConfig, repositories::policies::MariadbPolicies,
    services::policies::IAMPolicies,
};

#[derive(Clone)]
pub struct ServiceRegister {
    pub policies_service: DynPoliciesService,
}

impl ServiceRegister {
    pub fn new(pool: MySqlPool, config: Arc<AppConfig>) -> Self {
        let policies_repository = Arc::new(MariadbPolicies::new(pool));
        let policies_service = Arc::new(IAMPolicies::new(policies_repository));
        Self { policies_service }
    }
}
