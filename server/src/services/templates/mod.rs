use askama::Template;
use serde::Deserialize;

pub struct ConnectorInfo {
    pub name: String,
    pub logo: String,
    pub url: String,
}

#[derive(Template, Default)]
#[template(path = "porta.html")]
pub struct Porta {
    pub req_path: String,
    pub prompt: String,
    pub post_url: String,
    pub username: String,
    pub invalid: bool,
    pub connectors: Vec<ConnectorInfo>,
}

#[derive(Debug, Deserialize)]
pub struct Connector {
    pub client_id: Option<String>,
    pub invalid: Option<bool>,
    pub subject: Option<String>,
}

#[derive(Template, Default)]
#[template(path = "approval.html")]
pub struct Approval {
    pub req_path: String,
    pub client: String,
    pub scopes: Vec<String>,
    pub auth_req_id: String,
}

#[derive(Debug, Deserialize)]
pub struct ApprovalInfo {
    pub hmac: String,
    pub req: String,
    pub approval: Option<String>,
}
