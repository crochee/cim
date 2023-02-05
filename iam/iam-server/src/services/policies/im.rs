use async_trait::async_trait;

use cim_core::Result;

use crate::{
    models::{List, ID},
    repo::{
        policies::{Content, Opts},
        DynRepository,
    },
};

use super::Policy;

pub struct IAMPolicies {
    repository: DynRepository,
}

impl IAMPolicies {
    pub fn new(repository: DynRepository) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl super::PoliciesService for IAMPolicies {
    async fn create(&self, content: &Content) -> Result<ID> {
        self.repository.policy().create(None, content).await
    }

    async fn put(&self, id: &str, content: &Content) -> Result<()> {
        let found = self
            .repository
            .policy()
            .exist(id, content.account_id.clone(), true)
            .await?;
        if found {
            return self
                .repository
                .policy()
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
        self.repository
            .policy()
            .create(Some(id.to_owned()), content)
            .await?;
        Ok(())
    }

    async fn get(
        &self,
        id: &str,
        account_id: Option<String>,
    ) -> Result<Policy> {
        self.repository.policy().get(id, account_id).await
    }

    async fn delete(&self, id: &str, account_id: Option<String>) -> Result<()> {
        self.repository.policy().delete(id, account_id).await
    }

    async fn list(&self, filter: &super::Querys) -> Result<List<Policy>> {
        self.repository.policy().list(filter).await
    }
}
