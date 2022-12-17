use async_trait::async_trait;

use cim_core::Result;

use crate::{
    models::{List, ID},
    repositories::policies::{Content, DynPoliciesRepository, Opts},
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
    async fn create(&self, content: &Content) -> Result<ID> {
        self.repository.create(None, content).await
    }

    async fn put(&self, id: &str, content: &Content) -> Result<()> {
        let found = self
            .repository
            .exist(id, content.account_id.clone(), true)
            .await?;
        if found {
            return self
                .repository
                .update(
                    id,
                    content.account_id.clone(),
                    &Opts {
                        desc: Some(content.desc.clone()),
                        version: Some(content.version.clone()),
                        statement: Some(content.statement.clone()),
                        unscoped: Some(true),
                    },
                )
                .await;
        }
        self.repository.create(Some(id.to_owned()), content).await?;
        Ok(())
    }

    async fn get(
        &self,
        id: &str,
        account_id: Option<String>,
    ) -> Result<Policy> {
        self.repository.get(id, account_id).await
    }

    async fn delete(&self, id: &str, account_id: Option<String>) -> Result<()> {
        self.repository.delete(id, account_id).await
    }

    async fn list(&self, filter: &super::Querys) -> Result<List<Policy>> {
        self.repository.list(filter).await
    }
}
