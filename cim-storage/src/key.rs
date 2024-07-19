use jsonwebkey as jwk;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema, PartialEq, Clone)]
pub struct Keys {
    pub id: String,
    pub signing_key: jwk::JsonWebKey,
    pub signing_key_pub: jwk::JsonWebKey,
    pub verification_keys: Vec<VerificationKey>,
    pub next_rotation: i64,
}

impl Default for Keys {
    fn default() -> Self {
        Self {
            id: String::new(),
            signing_key: jwk::JsonWebKey::new(jwk::Key::generate_p256()),
            signing_key_pub: jwk::JsonWebKey::new(jwk::Key::generate_p256()),
            verification_keys: vec![],
            next_rotation: 0,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema, PartialEq, Clone)]
pub struct VerificationKey {
    pub public_key: jwk::JsonWebKey,
    pub expiry: i64,
}

impl Default for VerificationKey {
    fn default() -> Self {
        Self {
            public_key: jwk::JsonWebKey::new(jwk::Key::generate_p256()),
            expiry: 0,
        }
    }
}
