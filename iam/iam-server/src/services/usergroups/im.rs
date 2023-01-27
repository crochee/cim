use async_trait::async_trait;

use cim_core::Result;

use crate::{
    models::{List, ID},
    repo::usergroups::{Content, DynUserGroupsRepository, Opts, Querys},
};

use super::{UserGroup, UserGroupsService};

pub struct IAMUserGroups {
    repository: DynUserGroupsRepository,
}

impl IAMUserGroups {
    pub fn new(repository: DynUserGroupsRepository) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl UserGroupsService for IAMUserGroups {
    async fn create(&self, content: &Content) -> Result<ID> {
        self.repository.create(None, content).await
    }
    async fn put(&self, id: &str, content: &Content) -> Result<()> {
        let found = self
            .repository
            .exist(id, Some(content.account_id.clone()), true)
            .await?;
        if found {
            return self
                .repository
                .update(
                    id,
                    Some(content.account_id.clone()),
                    &Opts {
                        name: Some(content.name.clone()),
                        desc: Some(content.desc.clone()),
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
    ) -> Result<UserGroup> {
        self.repository.get(id, account_id).await
    }
    async fn delete(&self, id: &str, account_id: Option<String>) -> Result<()> {
        self.repository.delete(id, account_id).await
    }
    async fn list(&self, filter: &Querys) -> Result<List<UserGroup>> {
        self.repository.list(filter).await
    }
}
