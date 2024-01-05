use serde_json::value::RawValue;

/// Identity represents the ID Token claims supported by the server.
#[derive(Debug, Default)]
pub struct Identity {
    pub user_id: String,
    pub username: String,
    pub preferred_username: String,
    pub email: Option<String>,
    pub email_verified: bool,
    pub mobile: Option<String>,
    pub groups: Vec<String>,
    /// ConnectorData holds data used by the connector for subsequent requests after initial
    /// authentication, such as access tokens for upstream provides.
    ///
    /// This data is never shared with end users, OAuth clients, or through the API.
    pub connector_data: Box<RawValue>,
}
