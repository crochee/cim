mod im;

use std::sync::Arc;

use async_trait::async_trait;

use cim_core::Result;

use crate::{
    models::{usergroupbinding::UserGroupBinding, List},
    repo::usergroupbindings::{Content, Opts, Querys},
};

pub use im::IAMUserGroupBindings;

pub type DynUserGroupBindingsService =
    Arc<dyn UserGroupBindingsService + Send + Sync>;

#[async_trait]
pub trait UserGroupBindingsService {
    async fn create(&self, acount_id: String, content: &Content) -> Result<()>;

    async fn delete(&self, opts: &Opts) -> Result<()>;

    async fn list(&self, filter: &Querys) -> Result<List<UserGroupBinding>>;
}
