mod userpassword;

use async_trait::async_trait;
use axum::extract::Request;
use cim::errors;
use mockall::automock;
use serde_json::value::RawValue;

use cim_slo::Result;
use cim_storage::Claim;

pub use userpassword::UserPassword;

/// CallbackConnector is an interface implemented by connectors which use an OAuth
/// style redirect flow to determine user information.
/// The scopes requested by the client.
/// openid必需：表示请求使用 OpenID Connect 协议进行身份验证。必须包含此作用域。
/// profile请求访问用户的默认配置文件信息，如姓名、昵称、性别等
/// email请求访问用户的电子邮件地址。
/// address请求访问用户的地址信息。
/// phone请求访问用户的电话号码。
/// offline_access请求获取 Refresh Token，以便在 Access Token 过期后刷新令牌。
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
        scopes: &Vec<String>,
        callback_url: &str,
        state: &str,
    ) -> Result<String>;

    /// Handle the callback to the server and return an identity.
    async fn handle_callback(
        &self,
        scopes: &Vec<String>,
        req: Request,
    ) -> Result<Identity>;

    fn support_refresh(&self) -> bool {
        false
    }

    async fn refresh(
        &self,
        _scopes: &Vec<String>,
        _identity: &Identity,
    ) -> Result<Identity> {
        Err(errors::unauthorized())
    }
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
