use async_trait::async_trait;

use cim_core::Result;

use crate::{
    models::{role::RoleBindings, List, ID},
    repo::{
        roles::{Content, Opts, Querys},
        DynRepository,
    },
};

use super::Role;

pub struct IAMRoles {
    repository: DynRepository,
}

impl IAMRoles {
    pub fn new(repository: DynRepository) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl super::RolesService for IAMRoles {
    async fn create(&self, content: &Content) -> Result<ID> {
        self.repository.role().create(None, content).await
    }
    async fn put(&self, id: &str, content: &Content) -> Result<()> {
        let found = self
            .repository
            .role()
            .exist(id, Some(content.account_id.clone()), true)
            .await?;
        if found {
            return self
                .repository
                .role()
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
        self.repository
            .role()
            .create(Some(id.to_owned()), content)
            .await?;
        Ok(())
    }
    async fn get(&self, id: &str, filter: &Querys) -> Result<RoleBindings> {
        self.repository.role().get(id, filter).await
    }
    async fn delete(&self, id: &str, account_id: Option<String>) -> Result<()> {
        self.repository.role().delete(id, account_id).await
    }
    async fn list(&self, filter: &Querys) -> Result<List<Role>> {
        self.repository.role().list(filter).await
    }
    async fn add_user(
        &self,
        id: &str,
        account_id: &str,
        user_id: &str,
    ) -> Result<()> {
        self.repository
            .role()
            .add_user(id, account_id, user_id)
            .await
    }
    async fn delete_user(&self, id: &str, user_id: &str) -> Result<()> {
        self.repository.role().delete_user(id, user_id).await
    }
    async fn add_policy(
        &self,
        id: &str,
        account_id: &str,
        policy_id: &str,
    ) -> Result<()> {
        self.repository
            .role()
            .add_policy(id, account_id, policy_id)
            .await
    }
    async fn delete_policy(&self, id: &str, policy_id: &str) -> Result<()> {
        self.repository.role().delete_policy(id, policy_id).await
    }
}
