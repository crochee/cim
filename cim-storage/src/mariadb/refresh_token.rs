use async_trait::async_trait;
use serde_json::value::RawValue;
use sqlx::{types::Json, MySqlPool, Row};

use cim_slo::{errors, Result};
use cim_watch::{WatchGuard, Watcher};

use crate::{refresh_token::RefreshToken, Claim, Event, Interface, List};

#[derive(Clone)]
pub struct RefreshTokenImpl {
    pool: MySqlPool,
}

impl RefreshTokenImpl {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Interface for RefreshTokenImpl {
    type T = RefreshToken;
    type L = ();
    async fn put(&self, content: &Self::T, _ttl: u64) -> Result<()> {
        sqlx::query(
            r#"REPLACE INTO `refresh_token`
            (`id`,`client_id`,`scopes`,`nonce`,`token`,`obsolete_token`,
            `claim`,`connector_id`,`connector_data`,`last_used_at`)
            VALUES(?,?,?,?,?,?,?,?,?,?);"#,
        )
        .bind(&content.id)
        .bind(&content.client_id)
        .bind(Json(&content.scopes))
        .bind(&content.nonce)
        .bind(&content.token)
        .bind(&content.obsolete_token)
        .bind(Json(&content.claim))
        .bind(&content.connector_id)
        .bind(content.connector_data.as_ref().map(|v| v.to_string()))
        .bind(content.last_used_at)
        .execute(&self.pool)
        .await
        .map_err(errors::any)?;
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<()> {
        sqlx::query(
            r#"UPDATE `refresh_token` SET `deleted` = `id`,`deleted_at`= now()
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
            r#"SELECT `id`,`client_id`,`scopes`,`nonce`,`token`,`obsolete_token`,
            `claim`,`connector_id`,`connector_data`,`last_used_at`
            FROM `refresh_token`
            WHERE id = ? AND deleted = 0;"#,
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
        output.client_id = row.try_get("client_id").map_err(errors::any)?;
        output.scopes = row
            .try_get::<Json<Vec<String>>, _>("scopes")
            .map_err(errors::any)?
            .0;
        output.nonce = row.try_get("nonce").map_err(errors::any)?;
        output.token = row.try_get("token").map_err(errors::any)?;
        output.obsolete_token =
            row.try_get("obsolete_token").map_err(errors::any)?;
        output.claim = row
            .try_get::<Json<Claim>, _>("claim")
            .map_err(errors::any)?
            .0;
        output.connector_id =
            row.try_get("connector_id").map_err(errors::any)?;
        output.connector_data = row
            .try_get::<Option<String>, _>("connector_data")
            .map_err(errors::any)?
            .map(|v| RawValue::from_string(v).unwrap());
        output.created_at = row.try_get("created_at").map_err(errors::any)?;
        output.updated_at = row.try_get("updated_at").map_err(errors::any)?;
        output.last_used_at =
            row.try_get("last_used_at").map_err(errors::any)?;
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
    ) -> Box<dyn WatchGuard + Send> {
        todo!()
    }
    async fn count(&self, _opts: &Self::L, _unscoped: bool) -> Result<i64> {
        todo!()
    }
}
