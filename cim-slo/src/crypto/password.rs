use argon2::Config;
use rand::Rng;

use crate::{errors, Result};

pub fn encrypt(password: &str, secret: &str) -> Result<String> {
    let password_salt = rand::rng()
        .sample_iter(&rand::distr::Alphanumeric)
        .take(255)
        .map(char::from)
        .collect::<String>();
    let cfg = Config {
        secret: secret.as_bytes(),
        ..Default::default()
    };
    argon2::hash_encoded(password.as_bytes(), password_salt.as_bytes(), &cfg)
        .map_err(errors::any)
}

pub fn verify(encoded: &str, pwd: &str, secret: &str) -> Result<bool> {
    argon2::verify_encoded_ext(encoded, pwd.as_bytes(), secret.as_bytes(), &[])
        .map_err(errors::any)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_verify() {
        let password = String::from("ag1234567890123456789");
        let secret = rand::rng()
            .sample_iter(&rand::distr::Alphanumeric)
            .take(64)
            .map(char::from)
            .collect::<String>();
        let h = encrypt(&password, &secret).unwrap();
        assert_eq!(h.len(), 415);
        assert!(verify(&h, &password, &secret).unwrap())
    }
}
