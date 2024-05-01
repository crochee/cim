use async_trait::async_trait;
use serde_json::value::RawValue;
use sqlx::{MySqlPool, Row};

use cim_slo::{errors, next_id, Result};

use crate::ID;

use super::{RefreshToken, RefreshTokenStore};

pub struct RefreshTokenImpl {
    pool: MySqlPool,
}

impl RefreshTokenImpl {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RefreshTokenStore for RefreshTokenImpl {
    async fn put_refresh_token(&self, content: &RefreshToken) -> Result<ID> {
        let id = if content.id.is_empty() {
            next_id().map_err(errors::any)?
        } else {
            content
                .id
                .parse::<u64>()
                .map_err(|err| errors::bad_request(&err))?
        };

        let scopes =
            serde_json::to_string(&content.scopes).map_err(errors::any)?;

        let claim =
            serde_json::to_string(&content.claim).map_err(errors::any)?;

        sqlx::query(
            r#"REPLACE INTO `refresh_token`
            (`id`,`client_id`,`scopes`,`nonce`,`token`,`obsolete_token`,
            `claim`,`connector_id`,`connector_data`,`last_used_at`)
            VALUES(?,?,?,?,?,?,?,?,?,?,?);"#,
        )
        .bind(id)
        .bind(&content.client_id)
        .bind(scopes)
        .bind(&content.nonce)
        .bind(&content.token)
        .bind(&content.obsolete_token)
        .bind(claim)
        .bind(&content.connector_id)
        .bind(content.connector_data.as_ref().map(|v| v.to_string()))
        .bind(content.last_used_at)
        .execute(&self.pool)
        .await
        .map_err(errors::any)?;
        Ok(ID { id: id.to_string() })
    }
    async fn get_refresh_token(&self, id: &str) -> Result<RefreshToken> {
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

        let scopes = serde_json::from_str(
            &row.try_get::<String, _>("scopes").map_err(errors::any)?,
        )
        .map_err(errors::any)?;

        let claim = serde_json::from_str(
            &row.try_get::<String, _>("claim").map_err(errors::any)?,
        )
        .map_err(errors::any)?;

        let connector_data = row
            .try_get::<Option<String>, _>("connector_data")
            .map_err(errors::any)?
            .map(|v| RawValue::from_string(v).unwrap());

        Ok(RefreshToken {
            id: row
                .try_get::<u64, _>("id")
                .map_err(errors::any)?
                .to_string(),
            client_id: row.try_get("client_id").map_err(errors::any)?,
            scopes,
            nonce: row.try_get("nonce").map_err(errors::any)?,
            token: row.try_get("token").map_err(errors::any)?,
            obsolete_token: row
                .try_get("obsolete_token")
                .map_err(errors::any)?,
            claim,
            connector_id: row.try_get("connector_id").map_err(errors::any)?,
            connector_data,
            created_at: row.try_get("created_at").map_err(errors::any)?,
            updated_at: row.try_get("updated_at").map_err(errors::any)?,
            last_used_at: row.try_get("last_used_at").map_err(errors::any)?,
        })
    }
    async fn delete_refresh_token(&self, id: &str) -> Result<()> {
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
}
