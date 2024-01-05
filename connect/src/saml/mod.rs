use anyhow::Result;
use async_trait::async_trait;
use mockall::automock;

use crate::identity::Identity;
use crate::scope::Scopes;

/// SAMLConnector represents SAML connectors which implement the HTTP POST binding.
///
///	RelayState is handled by the server.
///
/// See: https://docs.oasis-open.org/security/saml/v2.0/saml-bindings-2.0-os.pdf
/// "3.5 HTTP POST Binding"
#[automock]
#[async_trait]
pub trait SAMLConnector {
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
