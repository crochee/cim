use anyhow::Result;
use async_trait::async_trait;
use http::Request;
use mockall::automock;

use crate::identity::Identity;
use crate::scope::Scopes;

/// CallbackConnector is an interface implemented by connectors which use an OAuth
/// style redirect flow to determine user information.
#[automock]
#[async_trait]
pub trait CallbackConnector<B: Send + Sync> {
    /// The initial URL to redirect the user to.
    ///
    /// OAuth2 implementations should request different scopes from the upstream
    /// identity provider based on the scopes requested by the downstream client.
    /// For example, if the downstream client requests a refresh token from the
    /// server, the connector should also request a token from the provider.
    ///
    /// Many identity providers have arbitrary restrictions on refresh tokens. For
    /// example Google only allows a single refresh token per client/user/scopes
    /// combination, and wont return a refresh token even if offline access is
    /// requested if one has already been issues. There's no good general answer
    /// for these kind of restrictions, and may require this package to become more
    /// aware of the global set of user/connector interactions.
    async fn login_url(
        &self,
        s: &Scopes,
        callback_url: &str,
        state: &str,
    ) -> Result<String>;

    /// Handle the callback to the server and return an identity.
    async fn handle_callback(
        &self,
        s: &Scopes,
        req: Request<B>,
    ) -> Result<Identity>;
}
