pub mod auth;
pub mod matcher;

use std::sync::Arc;

use async_trait::async_trait;

use cim_core::Result;

use crate::models::req::Request;

pub type DynAuthorizer = Arc<dyn Authorizer + Send + Sync>;

#[async_trait]
pub trait Authorizer {
    /// authorize return  ok or error
    async fn authorize(&self, input: &Request) -> Result<()>;
}

pub trait Matcher {
    fn matches(
        &self,
        delimiter_start: char,
        delimiter_end: char,
        haystack: Vec<String>,
        needle: &str,
    ) -> Result<bool>;
}
