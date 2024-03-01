use async_trait::async_trait;
use sqlx::{MySqlPool, Row};

use slo::{errors, Result};

use super::*;

pub struct KeyImpl {
    pool: MySqlPool,
    id: u64,
}

impl KeyImpl {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool, id: 1 }
    }
}

#[async_trait]
impl KeyStore for KeyImpl {
    async fn get_key(&self) -> Result<Keys> {
        let row = match sqlx::query(
            r#"SELECT `verification_keys`,`signing_key`,`signing_key_pub`,`next_rotation` 
            FROM `key`
            WHERE id = ?;"#,
        )
        .bind(self.id)
        .fetch_optional(&self.pool)
        .await
        {
            Ok(v) => match v {
                Some(value) => Ok(value),
                None => Err(errors::not_found("no rows")),
            },
            Err(err) => Err(errors::any(err)),
        }?;
        let signing_key: jwk::JsonWebKey = serde_json::from_str(
            &row.try_get::<String, _>("signing_key")
                .map_err(errors::any)?,
        )
        .map_err(errors::any)?;
        let signing_key_pub: jwk::JsonWebKey = serde_json::from_str(
            &row.try_get::<String, _>("signing_key_pub")
                .map_err(errors::any)?,
        )
        .map_err(errors::any)?;
        let verification_keys: Vec<VerificationKey> = serde_json::from_str(
            &row.try_get::<String, _>("verification_keys")
                .map_err(errors::any)?,
        )
        .map_err(errors::any)?;
        Ok(Keys {
            signing_key,
            signing_key_pub,
            verification_keys,
            next_rotation: row
                .try_get::<u64, _>("next_rotation")
                .map_err(errors::any)? as i64,
        })
    }
    async fn update_key(&self, nk: &Keys) -> Result<()> {
        let verification_keys = serde_json::to_string(&nk.verification_keys)
            .map_err(errors::any)?;
        let signing_key =
            serde_json::to_string(&nk.signing_key).map_err(errors::any)?;
        let signing_key_pub =
            serde_json::to_string(&nk.signing_key_pub).map_err(errors::any)?;
        sqlx::query(
            r#"UPDATE `key` SET
            `verification_keys` = ?,
            `signing_key` = ?,
            `signing_key_pub` = ?,
            `next_rotation` = ?
            WHERE `id` = ?;"#,
        )
        .bind(verification_keys)
        .bind(signing_key)
        .bind(signing_key_pub)
        .bind(nk.next_rotation as u64)
        .bind(self.id)
        .execute(&self.pool)
        .await
        .map_err(errors::any)?;

        Ok(())
    }
    async fn create_key(&self, nk: &Keys) -> Result<()> {
        let verification_keys = serde_json::to_string(&nk.verification_keys)
            .map_err(errors::any)?;
        let signing_key =
            serde_json::to_string(&nk.signing_key).map_err(errors::any)?;
        let signing_key_pub =
            serde_json::to_string(&nk.signing_key_pub).map_err(errors::any)?;
        sqlx::query(
            r#"INSERT INTO `key`
            (`id`,`verification_keys`,`signing_key`,`signing_key_pub`,`next_rotation`)
            VALUES(?,?,?,?,?);"#,
        )
        .bind(self.id)
        .bind(verification_keys)
        .bind(signing_key)
        .bind(signing_key_pub)
        .bind(nk.next_rotation as u64)
        .execute(&self.pool)
        .await
        .map_err(errors::any)?;

        Ok(())
    }
}
