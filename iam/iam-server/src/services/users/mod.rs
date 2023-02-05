mod im;

use std::sync::Arc;

use async_trait::async_trait;

use cim_core::Result;

use crate::{
    models::{user::User, List, ID},
    repo::users::{Content, Querys},
};

pub use im::IAMUsers;

pub type DynUsers = Arc<dyn UsersService + Send + Sync>;

#[async_trait]
pub trait UsersService {
    async fn create(&self, content: &Content) -> Result<ID>;
    async fn put(&self, id: &str, content: &Content) -> Result<()>;
    async fn get(&self, id: &str, account_id: Option<String>) -> Result<User>;
    async fn delete(&self, id: &str, account_id: Option<String>) -> Result<()>;
    async fn list(&self, filter: &Querys) -> Result<List<User>>;
}
