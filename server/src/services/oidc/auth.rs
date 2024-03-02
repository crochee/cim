use http::Uri;
use serde::{Deserialize, Serialize};
use slo::{errors, Result};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
pub struct AuthRequest {
    pub response_type: String,
    pub client_id: String,
    pub redirect_uri: Option<String>,
    pub scope: Option<String>,
    pub state: Option<String>,
    pub connector_id: Option<String>,
}

pub async fn auth(req: &mut AuthRequest) -> Result<String> {
    let mut connector = String::from("/v1/login");
    let uri = connector.parse::<Uri>().map_err(errors::any)?;
    if uri.query().is_none() {
        connector.push('?');
    } else {
        connector.push('&');
    }
    connector.push_str("back=/auth&");
    req.connector_id = None;
    connector.push_str(&serde_urlencoded::to_string(req).map_err(errors::any)?);
    Ok(connector)
}
