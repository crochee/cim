use http::Uri;
use serde::{Deserialize, Serialize};
use slo::{errors, Result};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
pub struct AuthRequest {
    #[validate(length(min = 1, max = 255))]
    pub response_type: String,
    #[validate(length(min = 1, max = 255))]
    pub client_id: String,
    #[validate(url)]
    pub redirect_uri: String,
    pub scope: Option<String>,
    #[validate(length(min = 1, max = 255))]
    pub state: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub connector_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub back: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_approval: Option<bool>,
}

pub async fn auth(req: &mut AuthRequest) -> Result<String> {
    let mut connector = String::from("/login");
    if let Some(connector_id) = &req.connector_id {
        connector.push('/');
        connector.push_str(connector_id);
    }
    let uri = connector.parse::<Uri>().map_err(errors::any)?;
    if uri.query().is_none() {
        connector.push('?');
    } else {
        connector.push('&');
    }
    req.back = Some("/auth".to_string());
    req.connector_id = None;
    connector.push_str(&serde_urlencoded::to_string(req).map_err(errors::any)?);
    Ok(connector)
}

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
pub struct ReqHmac {
    pub req: String,
    pub hmac: String,
}
