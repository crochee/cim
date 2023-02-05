use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Role {
    pub id: String,
    pub account_id: String,
    pub user_id: String,
    pub name: String,
    pub desc: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RoleBindings {
    pub id: String,
    pub account_id: String,
    pub user_id: String,
    pub name: String,
    pub desc: String,
    pub links: Vec<RoleBinding>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RoleBinding {
    pub kind: Kind,
    pub subject_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Hash, PartialEq, Eq)]
pub enum Kind {
    User,
    Policy,
}
