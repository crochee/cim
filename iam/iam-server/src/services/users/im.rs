use async_trait::async_trait;

use cim_core::Result;

use crate::{
    models::{List, ID},
    repo::{
        users::{Content, Opts, Querys},
        DynRepository,
    },
};

use super::User;

pub struct IAMUsers {
    repository: DynRepository,
}

impl IAMUsers {
    pub fn new(repository: DynRepository) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl super::UsersService for IAMUsers {
    async fn create(&self, content: &Content) -> Result<ID> {
        self.repository.user().create(None, content).await
    }
    async fn put(&self, id: &str, content: &Content) -> Result<()> {
        let found = self
            .repository
            .user()
            .exist(id, content.account_id.clone(), true)
            .await?;
        if found {
            return self
                .repository
                .user()
                .update(
                    id,
                    content.account_id.clone(),
                    &Opts {
                        name: Some(content.name.clone()),
                        nick_name: content.nick_name.clone(),
                        desc: Some(content.desc.clone()),
                        email: content.email.clone(),
                        mobile: content.mobile.clone(),
                        sex: content.sex.clone(),
                        image: content.image.clone(),
                        password: Some(content.password.clone()),
                        unscoped: Some(true),
                    },
                )
                .await;
        }
        self.repository
            .user()
            .create(Some(id.to_owned()), content)
            .await?;
        Ok(())
    }
    async fn get(&self, id: &str, account_id: Option<String>) -> Result<User> {
        self.repository.user().get(id, account_id).await
    }
    async fn delete(&self, id: &str, account_id: Option<String>) -> Result<()> {
        self.repository.user().delete(id, account_id).await
    }
    async fn list(&self, filter: &Querys) -> Result<List<User>> {
        self.repository.user().list(filter).await
    }
}
