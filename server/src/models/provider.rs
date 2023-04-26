use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Provider {
    pub id: String,
    pub secret: String,
    pub redirect_url: String,
    pub name: String,
    pub prompt: String,
    pub logo_url: String,
    pub refresh: bool,
}
