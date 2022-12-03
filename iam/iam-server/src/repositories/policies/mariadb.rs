use anyhow::Context;
use async_trait::async_trait;
use sqlx::MySqlPool;

#[derive(Clone)]
pub struct MariadbPolicies {
    pool: MySqlPool,
}

impl MariadbPolicies {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl super::PoliciesRepository for MariadbPolicies {}
