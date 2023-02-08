use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Keys {
    pub signing_key: KeyValue,
    pub verification_keys: Vec<VerificationKey>,
    pub next_rotation: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VerificationKey {
    pub value: KeyValue,
    pub expiry: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct KeyValue {
    pub id: String,
    pub value: String,
    pub alg: String,
}
