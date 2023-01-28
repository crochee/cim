use async_trait::async_trait;

use cim_core::Result;

use crate::{
    models::List,
    repo::usergroupbindings::{
        Content, DynUserGroupBindingsRepository, Opts, Querys,
    },
};

use super::{UserGroupBinding, UserGroupBindingsService};

pub struct IAMUserGroupBindings {
    repository: DynUserGroupBindingsRepository,
}

impl IAMUserGroupBindings {
    pub fn new(repository: DynUserGroupBindingsRepository) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl UserGroupBindingsService for IAMUserGroupBindings {
    async fn create(&self, acount_id: String, content: &Content) -> Result<()> {
        self.repository.create(acount_id, content).await
    }

    async fn delete(&self, opts: &Opts) -> Result<()> {
        self.repository.delete(opts).await
    }

    async fn list(&self, filter: &Querys) -> Result<List<UserGroupBinding>> {
        self.repository.list(filter).await
    }
}
