mod userpassword;

use async_trait::async_trait;
use axum::extract::Request;
use cim::errors;
use mockall::automock;
use serde::Deserialize;
use serde_json::value::RawValue;

use cim_slo::Result;
use cim_storage::Claim;

pub use userpassword::UserPassword;

/// CallbackConnector is an interface implemented by connectors which use an OAuth
/// style redirect flow to determine user information.
#[automock]
#[async_trait]
pub trait CallbackConnector: Send + Sync {
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
        req: Request,
    ) -> Result<Identity>;

    fn support_refresh(&self) -> bool {
        false
    }

    async fn refresh(
        &self,
        _s: &Scopes,
        _identity: &Identity,
    ) -> Result<Identity> {
        Err(errors::unauthorized())
    }
}

/// Scopes represents additional data requested by the clients about the end user.
#[derive(Debug, Deserialize, Default)]
pub struct Scopes {
    /// The client has requested a refresh token from the server.
    pub offline_access: bool,
    /// The client has requested group information about the end user.
    pub groups: bool,
}

/// Identity represents the ID Token claims supported by the server.
#[derive(Debug, Default)]
pub struct Identity {
    pub claim: Claim,
    /// ConnectorData holds data used by the connector for subsequent requests after initial
    /// authentication, such as access tokens for upstream provides.
    ///
    /// This data is never shared with end users, OAuth clients, or through the API.
    pub connector_data: Option<Box<RawValue>>,
}

pub fn parse_scopes(scopes: &Vec<String>) -> Scopes {
    let mut s = Scopes::default();
    for scope in scopes {
        if scope.eq("offline_access") {
            s.offline_access = true;
        } else if scope.eq("groups") {
            s.groups = true;
        }
    }
    s
}
