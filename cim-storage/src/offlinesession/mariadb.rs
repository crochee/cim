use async_trait::async_trait;
use serde_json::value::RawValue;
use sqlx::{MySqlPool, Row};

use cim_slo::{errors, Result};

use super::{OfflineSession, OfflineSessionStore};

pub struct OfflineSessionImpl {
    pool: MySqlPool,
}

impl OfflineSessionImpl {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OfflineSessionStore for OfflineSessionImpl {
    async fn put_offline_session(
        &self,
        content: &OfflineSession,
    ) -> Result<()> {
        let refresh =
            serde_json::to_string(&content.refresh).map_err(errors::any)?;

        sqlx::query(
            r#"REPLACE INTO `offline_session`
            (`user_id`,`conn_id`,`refresh`)
            VALUES(?,?,?);"#,
        )
        .bind(&content.user_id)
        .bind(&content.conn_id)
        .bind(refresh)
        .execute(&self.pool)
        .await
        .map_err(errors::any)?;
        Ok(())
    }
    async fn get_offline_session(
        &self,
        user_id: &str,
        conn_id: &str,
    ) -> Result<OfflineSession> {
        let row = match sqlx::query(
            r#"SELECT `user_id`,`conn_id`,`refresh`
            FROM `offline_session`
            WHERE id = ? AND deleted = 0;"#,
        )
        .bind(user_id)
        .bind(conn_id)
        .fetch_optional(&self.pool)
        .await
        {
            Ok(v) => match v {
                Some(value) => Ok(value),
                None => Err(errors::not_found("no rows")),
            },
            Err(err) => Err(errors::any(err)),
        }?;
        let refresh = serde_json::from_str(
            &row.try_get::<String, _>("refresh").map_err(errors::any)?,
        )
        .map_err(errors::any)?;
        let connector_data = row
            .try_get::<Option<String>, _>("connector_data")
            .map_err(errors::any)?
            .map(|v| RawValue::from_string(v).unwrap());

        Ok(OfflineSession {
            user_id: row
                .try_get::<String, _>("user_id")
                .map_err(errors::any)?,
            conn_id: row
                .try_get::<String, _>("conn_id")
                .map_err(errors::any)?,
            refresh,
            connector_data,
        })
    }
    async fn delete_offline_session(
        &self,
        user_id: &str,
        conn_id: &str,
    ) -> Result<()> {
        sqlx::query(
            r#"UPDATE `offline_session` SET `deleted` = `id`,`deleted_at`= now()
            WHERE user_id = ? AND conn_id = ? AND `deleted` = 0;"#,
        )
        .bind(user_id)
        .bind(conn_id)
        .execute(&self.pool)
        .await
        .map_err(errors::any)?;
        Ok(())
    }
}
