use serde::{Deserialize, Serialize};
use validator::Validate;

use cim_slo::{errors, Result};
use cim_storage::{
    connector::{Connector, ListParams},
    Interface,
};

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct AuthRequest {
    #[validate(length(min = 1, max = 255))]
    pub response_type: String,
    #[validate(length(min = 1, max = 255))]
    pub client_id: String,
    #[validate(url)]
    pub redirect_uri: String,
    pub scope: String,
    #[validate(length(min = 1, max = 255))]
    pub state: String,
    #[validate(length(min = 1, max = 255))]
    pub nonce: String,

    #[validate(length(min = 1))]
    pub code_challenge: String,
    pub code_challenge_method: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(length(min = 1))]
    pub connector_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_approval: Option<bool>,
}

pub async fn auth<S: Interface<T = Connector, L = ListParams>>(
    connector_store: &S,
    req: &mut AuthRequest,
) -> Result<String> {
    let mut connector = String::from("/connectors");
    if let Some(connector_id) = &req.connector_id {
        let mut connector_data = Connector {
            id: connector_id.to_owned(),
            ..Default::default()
        };
        connector_store.get(&mut connector_data).await?;
        connector.push('/');
        connector.push_str(&connector_data.id);
    }
    connector.push('&');
    req.connector_id = None;
    connector.push_str(&serde_urlencoded::to_string(req).map_err(errors::any)?);
    Ok(connector)
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct AuthRequestState {
    pub state: String,
    pub callback: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct ReqHmac {
    pub req: String,
    pub hmac: String,
    pub approval: Option<String>,
}
