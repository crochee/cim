use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct UserGroupBinding {
    #[serde(skip_deserializing)]
    pub id: String,
    pub user_group_id: String,
    pub kind: Kind,
    pub subject_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Hash, PartialEq, Eq)]
pub enum Kind {
    User,
    Role,
}
