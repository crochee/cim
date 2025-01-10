use async_trait::async_trait;
use jsonwebkey::JsonWebKey;
use sqlx::{types::Json, MySqlPool, Row};

use cim_slo::{errors, Result};

use crate::{
    key::{Keys, VerificationKey},
    Interface, List,
};

#[derive(Clone, Debug)]
pub struct KeysImpl {
    pool: MySqlPool,
}

impl KeysImpl {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Interface for KeysImpl {
    type T = Keys;
    type L = ();

    #[tracing::instrument]
    async fn put(&self, nk: &Self::T) -> Result<()> {
        sqlx::query(
            r#"REPLACE INTO `key`
            (`id`,`verification_keys`,`signing_key`,`signing_key_pub`,`next_rotation`)
            VALUES(?,?,?,?,?);"#,
        )
        .bind(&nk.id)
        .bind(Json(&nk.verification_keys))
        .bind(Json(&nk.signing_key))
        .bind(Json(&nk.signing_key_pub))
        .bind(nk.next_rotation as u64)
        .execute(&self.pool)
        .await
        .map_err(errors::any)?;

        Ok(())
    }

    #[tracing::instrument]
    async fn delete(&self, input: &Self::T) -> Result<()> {
        let id = input
            .id
            .parse::<u64>()
            .map_err(|err| errors::bad_request(&err))?;
        sqlx::query(
            r#"UPDATE `key` SET `deleted` = `id`,`deleted_at`= now()
            WHERE id = ? AND `deleted` = 0;"#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(errors::any)?;
        Ok(())
    }

    #[tracing::instrument]
    async fn get(&self, output: &mut Self::T) -> Result<()> {
        let id = output
            .id
            .parse::<u64>()
            .map_err(|err| errors::bad_request(&err))?;
        let row = match sqlx::query(
            r#"SELECT `id`,`verification_keys`,`signing_key`,`signing_key_pub`,`next_rotation`
            FROM `key`
            WHERE id = ?;"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        {
            Ok(v) => match v {
                Some(value) => Ok(value),
                None => Err(errors::not_found("no rows")),
            },
            Err(err) => Err(errors::any(err)),
        }?;
        output.id = row
            .try_get::<u64, _>("id")
            .map_err(errors::any)?
            .to_string();
        output.signing_key = row
            .try_get::<Json<JsonWebKey>, _>("signing_key")
            .map_err(errors::any)?
            .0;
        output.signing_key_pub = row
            .try_get::<Json<JsonWebKey>, _>("signing_key_pub")
            .map_err(errors::any)?
            .0;
        output.verification_keys = row
            .try_get::<Json<Vec<VerificationKey>>, _>("verification_keys")
            .map_err(errors::any)?
            .0;
        output.next_rotation = row
            .try_get::<u64, _>("next_rotation")
            .map_err(errors::any)? as i64;

        Ok(())
    }

    #[tracing::instrument]
    async fn list(
        &self,
        _opts: &Self::L,
        output: &mut List<Self::T>,
    ) -> Result<()> {
        let mut wheres = String::new();
        if !wheres.is_empty() {
            wheres.push_str(" AND ");
        }
        wheres.push_str(r#"`deleted` = 0"#);
        let rows = sqlx::query(
            format!(
                r#"SELECT `id`,`verification_keys`,`signing_key`,`signing_key_pub`,`next_rotation`
                FROM `key`
                WHERE {};"#,
                wheres,
            )
            .as_str(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(errors::any)?;
        for row in rows.iter() {
            output.data.push(Self::T {
                id: row
                    .try_get::<u64, _>("id")
                    .map_err(errors::any)?
                    .to_string(),
                signing_key: row
                    .try_get::<Json<JsonWebKey>, _>("signing_key")
                    .map_err(errors::any)?
                    .0,
                signing_key_pub: row
                    .try_get::<Json<JsonWebKey>, _>("signing_key_pub")
                    .map_err(errors::any)?
                    .0,
                verification_keys: row
                    .try_get::<Json<Vec<VerificationKey>>, _>(
                        "verification_keys",
                    )
                    .map_err(errors::any)?
                    .0,
                next_rotation: row
                    .try_get::<u64, _>("next_rotation")
                    .map_err(errors::any)?
                    as i64,
            });
        }
        Ok(())
    }

    async fn count(&self, _opts: &Self::L, _unscoped: bool) -> Result<i64> {
        todo!()
    }
}
