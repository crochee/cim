mod im;

use std::sync::Arc;

use async_trait::async_trait;

use cim_core::Result;

use crate::{
    models::{rolebinding::RoleBinding, List},
    repo::rolebindings::{Content, Opts, Querys},
};

pub use im::IAMRoleBindings;

pub type DynRoleBindingsService = Arc<dyn RoleBindingsService + Send + Sync>;

#[async_trait]
pub trait RoleBindingsService {
    async fn create(&self, acount_id: String, content: &Content) -> Result<()>;

    async fn delete(&self, opts: &Opts) -> Result<()>;

    async fn list(&self, filter: &Querys) -> Result<List<RoleBinding>>;
}
