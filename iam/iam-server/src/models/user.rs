use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub nick_name: String,
    pub desc: String,
    pub email: Option<String>,
    pub mobile: Option<String>,
    pub sex: Option<String>,
    pub image: Option<String>,
}
