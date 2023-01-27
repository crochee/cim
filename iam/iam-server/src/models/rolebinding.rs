use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct RoleBinding {
    #[serde(skip_deserializing)]
    pub id: String,
    pub role_id: String,
    pub kind: Kind,
    pub subject_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Hash, PartialEq, Eq)]
pub enum Kind {
    User,
    Policy,
}
