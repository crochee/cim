use async_trait::async_trait;

use cim_core::Result;

use crate::{
    models::{usergroup::UserGroupBindings, List, ID},
    repo::{
        usergroups::{Content, Opts, Querys},
        DynRepository,
    },
};

use super::{UserGroup, UserGroupsService};

pub struct IAMUserGroups {
    repository: DynRepository,
}

impl IAMUserGroups {
    pub fn new(repository: DynRepository) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl UserGroupsService for IAMUserGroups {
    async fn create(&self, content: &Content) -> Result<ID> {
        self.repository.user_group().create(None, content).await
    }
    async fn put(&self, id: &str, content: &Content) -> Result<()> {
        let found = self
            .repository
            .user_group()
            .exist(id, Some(content.account_id.clone()), true)
            .await?;
        if found {
            return self
                .repository
                .user_group()
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
            .user_group()
            .create(Some(id.to_owned()), content)
            .await?;
        Ok(())
    }
    async fn get(
        &self,
        id: &str,
        filter: &Querys,
    ) -> Result<UserGroupBindings> {
        self.repository.user_group().get(id, filter).await
    }
    async fn delete(&self, id: &str, account_id: Option<String>) -> Result<()> {
        self.repository.user_group().delete(id, account_id).await
    }
    async fn list(&self, filter: &Querys) -> Result<List<UserGroup>> {
        self.repository.user_group().list(filter).await
    }
    async fn add_user(
        &self,
        id: &str,
        account_id: &str,
        user_id: &str,
    ) -> Result<()> {
        self.repository
            .user_group()
            .add_user(id, account_id, user_id)
            .await
    }
    async fn delete_user(&self, id: &str, user_id: &str) -> Result<()> {
        self.repository.user_group().delete_user(id, user_id).await
    }
    async fn add_role(
        &self,
        id: &str,
        account_id: &str,
        role_id: &str,
    ) -> Result<()> {
        self.repository
            .user_group()
            .add_role(id, account_id, role_id)
            .await
    }
    async fn delete_role(&self, id: &str, role_id: &str) -> Result<()> {
        self.repository.user_group().delete_role(id, role_id).await
    }
}
