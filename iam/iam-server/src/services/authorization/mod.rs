mod auth;
pub mod matcher;

use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;

use cim_core::Result;

use crate::models::req::Request;

pub use auth::Auth as IAMAuth;

pub type DynAuthorizer = Arc<dyn Authorizer + Send + Sync>;

#[automock]
#[async_trait]
pub trait Authorizer {
    /// authorize return  ok or error
    async fn authorize(&self, input: &Request) -> Result<()>;
}

pub type DynMatcher = Arc<dyn Matcher + Send + Sync>;

pub trait Matcher {
    fn matches(
        &self,
        delimiter_start: char,
        delimiter_end: char,
        haystack: Vec<String>,
        needle: &str,
    ) -> Result<bool>;
}
