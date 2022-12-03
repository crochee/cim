use async_trait::async_trait;

use cim_core::Result;

use crate::{
    models::{List, Pagination, ID},
    repositories::policies::DynPoliciesRepository,
};

use super::Policy;

#[derive(Clone)]
pub struct IAMPolicies {
    repository: DynPoliciesRepository,
}

impl IAMPolicies {
    pub fn new(repository: DynPoliciesRepository) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl super::PoliciesService for IAMPolicies {
    async fn create(&self, policy: &Policy) -> Result<ID> {
        todo!()
    }
    async fn put(&self, id: &str, policy: &Policy) -> Result<()> {
        todo!()
    }
    async fn get(&self, id: &str) -> Result<Policy> {
        todo!()
    }
    async fn delete(&self, id: &str) -> Result<()> {
        todo!()
    }
    async fn list(&self, filter: &super::Filter) -> Result<List<Policy>> {
        Ok(List {
            data: Vec::new(),
            limit: 0,
            offset: 0,
            total: 0,
        })
    }
    async fn exist(&self, id: &str) -> Result<()> {
        todo!()
    }
}
