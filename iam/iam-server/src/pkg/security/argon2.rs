use argon2::{Config, ThreadMode};
use rand::Rng;

use cim_core::{Error, Result};

pub fn encrypt(password: &str, secret: &str) -> Result<String> {
    let password_salt = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(255)
        .map(char::from)
        .collect::<String>();
    let cfg = Config {
        secret: secret.as_bytes(),
        thread_mode: ThreadMode::Parallel,
        time_cost: 12,
        ..Default::default()
    };
    argon2::hash_encoded(password.as_bytes(), password_salt.as_bytes(), &cfg)
        .map_err(Error::any)
}

pub fn verify(encoded: &str, pwd: &str, secret: &str) -> Result<bool> {
    argon2::verify_encoded_ext(encoded, pwd.as_bytes(), secret.as_bytes(), &[])
        .map_err(Error::any)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_verify() {
        let password = String::from("io");
        let secret = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(64)
            .map(char::from)
            .collect::<String>();
        let h = encrypt(&password, &secret).unwrap();
        assert_eq!(h.len(), 414);
        assert!(verify(&h, &password, &secret).unwrap())
    }
}
