use crate::{errors, Result};
use sqlx::MySqlPool;

use crate::models::key::{KeyValue, Keys, VerificationKey};

pub async fn get(pool: &MySqlPool) -> Result<Keys> {
    match sqlx::query!(
        r#"SELECT `signing_key`,`verification_keys`,`next_rotation`
        FROM `key` WHERE `enable` = 1 AND `deleted` = 0;"#
    )
    .fetch_optional(pool)
    .await
    .map_err(errors::any)?
    {
        Some(v) => {
            let signing_key: KeyValue =
                serde_json::from_str(&v.signing_key).map_err(errors::any)?;
            let verification_keys: Vec<VerificationKey> =
                serde_json::from_str(&v.verification_keys)
                    .map_err(errors::any)?;
            Ok(Keys {
                signing_key,
                verification_keys,
                next_rotation: v.next_rotation,
            })
        }
        None => Err(errors::not_found("no rows")),
    }
}

pub async fn update(pool: &MySqlPool, nk: &Keys) -> Result<()> {
    let signing_key =
        serde_json::to_string(&nk.signing_key).map_err(errors::any)?;
    let verification_keys =
        serde_json::to_string(&nk.verification_keys).map_err(errors::any)?;
    sqlx::query!(
                r#"UPDATE `key` SET `signing_key` = ?,`verification_keys`= ?,`next_rotation` = ?
                WHERE `enable` = 1 AND `deleted` = 0;"#,
                signing_key,
                verification_keys,
                nk.next_rotation,
            )
            .execute(pool)
            .await
            .map_err(errors::any)?;
    Ok(())
}

pub async fn create(pool: &MySqlPool, nk: &Keys) -> Result<()> {
    let signing_key =
        serde_json::to_string(&nk.signing_key).map_err(errors::any)?;
    let verification_keys =
        serde_json::to_string(&nk.verification_keys).map_err(errors::any)?;
    sqlx::query!(
        r#"INSERT INTO `key`
            (`signing_key`,`verification_keys`,`next_rotation`,`enable`)
        VALUES(?,?,?,1);"#,
        signing_key,
        verification_keys,
        nk.next_rotation,
    )
    .execute(pool)
    .await
    .map_err(errors::any)?;
    Ok(())
}
