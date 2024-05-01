mod userpassword;

use async_trait::async_trait;
use axum::extract::Request;
use mockall::automock;
use serde::Deserialize;
use serde_json::value::RawValue;
use validator::Validate;

use cim_slo::{regexp::check_password, Result};
use cim_storage::Claim;

pub use userpassword::UserPassword;

#[automock]
#[async_trait]
pub trait PasswordConnector: Send + Sync {
    fn prompt(&self) -> &'static str;
    fn refresh_enabled(&self) -> bool;
    async fn login(&self, s: &Scopes, info: &Info) -> Result<Identity>;
    async fn refresh(
        &self,
        s: &Scopes,
        identity: &Identity,
    ) -> Result<Identity>;
}

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
}

/// SAMLConnector represents SAML connectors which implement the HTTP POST binding.
///
///	RelayState is handled by the server.
///
/// See: https://docs.oasis-open.org/security/saml/v2.0/saml-bindings-2.0-os.pdf
/// "3.5 HTTP POST Binding"
#[automock]
#[async_trait]
pub trait SAMLConnector: Send + Sync {
    /// POSTData returns an encoded SAML request and SSO URL for the server to
    /// render a POST form with.
    ///
    /// POSTData should encode the provided request ID in the returned serialized
    /// SAML request.
    async fn post_data(
        &self,
        s: &Scopes,
        request_id: &str,
    ) -> Result<(String, String)>;

    /// HandlePOST decodes, verifies, and maps attributes from the SAML response.
    /// It passes the expected value of the "InResponseTo" response field, which
    /// the connector must ensure matches the response value.
    ///
    /// See: https://www.oasis-open.org/committees/download.php/35711/sstc-saml-core-errata-2.0-wd-06-diff.pdf
    /// "3.2.2 Complex Type StatusResponseType"
    async fn handle_post(
        &self,
        s: &Scopes,
        saml_response: &str,
        in_response_to: &str,
    ) -> Result<Identity>;
}

/// Scopes represents additional data requested by the clients about the end user.
#[derive(Debug, Deserialize, Default)]
pub struct Scopes {
    /// The client has requested a refresh token from the server.
    pub offline_access: bool,
    /// The client has requested group information about the end user.
    pub groups: bool,
}

#[derive(Debug, Deserialize, Validate)]
pub struct Info {
    #[validate(length(min = 1))]
    pub subject: String,
    #[validate(custom = "check_password")]
    pub password: String,
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
