use async_trait::async_trait;

use cim_core::Result;

use crate::{
    models::List,
    repo::rolebindings::{Content, DynRoleBindingsRepository, Opts, Querys},
};

use super::{RoleBinding, RoleBindingsService};

pub struct IAMRoleBindings {
    repository: DynRoleBindingsRepository,
}

impl IAMRoleBindings {
    pub fn new(repository: DynRoleBindingsRepository) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl RoleBindingsService for IAMRoleBindings {
    async fn create(&self, acount_id: String, content: &Content) -> Result<()> {
        self.repository.create(acount_id, content).await
    }

    async fn delete(&self, opts: &Opts) -> Result<()> {
        self.repository.delete(opts).await
    }

    async fn list(&self, filter: &Querys) -> Result<List<RoleBinding>> {
        self.repository.list(filter).await
    }
}
