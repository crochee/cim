use anyhow::Result;
use async_trait::async_trait;
use mockall::automock;

use crate::identity::Identity;
use crate::scope::Scopes;

#[automock]
#[async_trait]
pub trait RefreshConnector {
    /// refresh is called when a client attempts to claim a refresh token. The
    /// connector should attempt to update the identity object to reflect any
    /// changes since the token was last refreshed.
    async fn refresh(
        &self,
        s: &Scopes,
        identity: &Identity,
    ) -> Result<Identity>;
}
