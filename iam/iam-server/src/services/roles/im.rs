use async_trait::async_trait;

use cim_core::Result;

use crate::{
    models::{List, ID},
    repo::roles::{Content, DynRolesRepository, Opts, Querys},
};

use super::Role;

pub struct IAMRoles {
    repository: DynRolesRepository,
}

impl IAMRoles {
    pub fn new(repository: DynRolesRepository) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl super::RolesService for IAMRoles {
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
    async fn get(&self, id: &str, account_id: Option<String>) -> Result<Role> {
        self.repository.get(id, account_id).await
    }
    async fn delete(&self, id: &str, account_id: Option<String>) -> Result<()> {
        self.repository.delete(id, account_id).await
    }
    async fn list(&self, filter: &Querys) -> Result<List<Role>> {
        self.repository.list(filter).await
    }
}
