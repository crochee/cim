use async_trait::async_trait;
use sqlx::{MySqlPool, Row};

use cim_slo::{errors, Result};
use cim_watch::Watcher;

use crate::{key::Keys, Event, Interface, List};

#[derive(Clone)]
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
    async fn put(&self, nk: &Self::T, _ttl: u64) -> Result<()> {
        let verification_keys = serde_json::to_string(&nk.verification_keys)
            .map_err(errors::any)?;
        let signing_key =
            serde_json::to_string(&nk.signing_key).map_err(errors::any)?;
        let signing_key_pub =
            serde_json::to_string(&nk.signing_key_pub).map_err(errors::any)?;
        sqlx::query(
            r#"REPLACE INTO `key`
            (`id`,`verification_keys`,`signing_key`,`signing_key_pub`,`next_rotation`)
            VALUES(?,?,?,?,?);"#,
        )
        .bind(&nk.id)
        .bind(verification_keys)
        .bind(signing_key)
        .bind(signing_key_pub)
        .bind(nk.next_rotation as u64)
        .execute(&self.pool)
        .await
        .map_err(errors::any)?;

        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<()> {
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
    async fn get(&self, id: &str, output: &mut Self::T) -> Result<()> {
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
        output.signing_key = serde_json::from_str(
            &row.try_get::<String, _>("signing_key")
                .map_err(errors::any)?,
        )
        .map_err(errors::any)?;
        output.signing_key_pub = serde_json::from_str(
            &row.try_get::<String, _>("signing_key_pub")
                .map_err(errors::any)?,
        )
        .map_err(errors::any)?;
        output.verification_keys = serde_json::from_str(
            &row.try_get::<String, _>("verification_keys")
                .map_err(errors::any)?,
        )
        .map_err(errors::any)?;
        output.next_rotation = row
            .try_get::<u64, _>("next_rotation")
            .map_err(errors::any)? as i64;

        Ok(())
    }
    async fn list(
        &self,
        _opts: &Self::L,
        _output: &mut List<Self::T>,
    ) -> Result<()> {
        todo!()
    }
    fn watch<W: Watcher<Event<Self::T>>>(
        &self,
        _handler: W,
        _remove: impl Fn() + Send + 'static,
    ) -> Box<dyn Fn() + Send> {
        todo!()
    }
    async fn count(&self, _opts: &Self::L, _unscoped: bool) -> Result<i64> {
        todo!()
    }
}
