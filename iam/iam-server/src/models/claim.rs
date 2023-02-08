use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub user_id: String,
    pub username: String,
    pub preferred_username: String,
    pub email: Option<String>,
    pub email_verified: bool,
    pub mobile: Option<String>,
    pub exp: Option<i64>,
}
