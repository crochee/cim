use async_trait::async_trait;
use serde_json::value::RawValue;
use sqlx::{MySqlPool, Row};

use slo::{errors, next_id, Result};

use crate::ID;

use super::{Connector, ConnectorStore};

pub struct ConnectorImpl {
    pool: MySqlPool,
}

impl ConnectorImpl {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ConnectorStore for ConnectorImpl {
    async fn put_connector(&self, content: &Connector) -> Result<ID> {
        let id = if content.id.is_empty() {
            next_id().map_err(errors::any)?
        } else {
            content
                .id
                .parse::<u64>()
                .map_err(|err| errors::bad_request(&err))?
        };
        sqlx::query(
            r#"REPLACE INTO `connector`
            (`id`,`type`,`name`,`resource_version`,`config`,`connector_data`)
            VALUES(?,?,?,?,?,?);"#,
        )
        .bind(id)
        .bind(&content.connector_type)
        .bind(&content.name)
        .bind(&content.response_version)
        .bind(&content.config)
        .bind(content.connector_data.as_ref().map(|v| v.to_string()))
        .execute(&self.pool)
        .await
        .map_err(errors::any)?;
        Ok(ID { id: id.to_string() })
    }
    async fn get_connector(&self, id: &str) -> Result<Connector> {
        let row = match sqlx::query(
            r#"SELECT `id`,`type`,`name`,`resource_version`,`config`,`connector_data`
            FROM `connector`
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
        let connector_data = row
            .try_get::<Option<String>, _>("connector_data")
            .map_err(errors::any)?
            .map(|v| RawValue::from_string(v).unwrap());

        Ok(Connector {
            id: row
                .try_get::<u64, _>("id")
                .map_err(errors::any)?
                .to_string(),
            connector_type: row.try_get("type").map_err(errors::any)?,
            name: row.try_get("name").map_err(errors::any)?,
            response_version: row.try_get("resource").map_err(errors::any)?,
            config: row.try_get("config").map_err(errors::any)?,
            connector_data,
        })
    }
    async fn delete_connector(&self, id: &str) -> Result<()> {
        sqlx::query(
            r#"UPDATE `connector` SET `deleted` = `id`,`deleted_at`= now()
            WHERE id = ? AND `deleted` = 0;"#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(errors::any)?;
        Ok(())
    }
}
