use anyhow::Result;
use async_trait::async_trait;
use mockall::automock;

use crate::identity::Identity;
use crate::scope::Scopes;

#[automock]
#[async_trait]
pub trait PasswordConnector {
    fn prompt(&self) -> &'static str;

    async fn login(
        &self,
        s: &Scopes,
        username: &str,
        password: &str,
    ) -> Result<(Identity, bool)>;
}
