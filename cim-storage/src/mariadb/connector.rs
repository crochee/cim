use async_trait::async_trait;
use serde_json::value::RawValue;
use sqlx::{MySqlPool, Row};

use cim_slo::{errors, Result};
use cim_watch::Watcher;

use crate::{connector::Connector, Event, Interface, List};

#[derive(Clone)]
pub struct ConnectorImpl {
    pool: MySqlPool,
}

impl ConnectorImpl {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Interface for ConnectorImpl {
    type T = Connector;
    type L = ();
    async fn put(&self, content: &Self::T, _ttl: u64) -> Result<()> {
        sqlx::query(
            r#"REPLACE INTO `connector`
            (`id`,`type`,`name`,`resource_version`,`config`,`connector_data`)
            VALUES(?,?,?,?,?,?);"#,
        )
        .bind(&content.id)
        .bind(&content.connector_type)
        .bind(&content.name)
        .bind(&content.response_version)
        .bind(&content.config)
        .bind(content.connector_data.as_ref().map(|v| v.to_string()))
        .execute(&self.pool)
        .await
        .map_err(errors::any)?;
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<()> {
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
    async fn get(&self, id: &str, output: &mut Self::T) -> Result<()> {
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
        output.connector_data = row
            .try_get::<Option<String>, _>("connector_data")
            .map_err(errors::any)?
            .map(|v| RawValue::from_string(v).unwrap());

        output.id = row
            .try_get::<u64, _>("id")
            .map_err(errors::any)?
            .to_string();
        output.connector_type = row.try_get("type").map_err(errors::any)?;
        output.name = row.try_get("name").map_err(errors::any)?;
        output.response_version =
            row.try_get("resource").map_err(errors::any)?;
        output.config = row.try_get("config").map_err(errors::any)?;
        Ok(())
    }
    async fn list(
        &self,
        _opts: &Self::L,
        output: &mut List<Self::T>,
    ) -> Result<()> {
        let rows = sqlx::query(
            r#"SELECT `id`,`type`,`name`,`resource_version`,`config`,`connector_data`
            FROM `connector`
            WHERE deleted = 0;"#,
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
                connector_type: row.try_get("type").map_err(errors::any)?,
                name: row.try_get("name").map_err(errors::any)?,
                response_version: row
                    .try_get("resource")
                    .map_err(errors::any)?,
                config: row.try_get("config").map_err(errors::any)?,
                connector_data: row
                    .try_get::<Option<String>, _>("connector_data")
                    .map_err(errors::any)?
                    .map(|v| RawValue::from_string(v).unwrap()),
            });
        }
        Ok(())
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
