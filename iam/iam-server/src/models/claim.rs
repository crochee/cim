use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub user_id: String,
    pub username: String,
    pub preferred_username: String,
    pub email: String,
    pub email_verified: bool,
    pub mobile: String,
    pub groups: Vec<String>,
}
