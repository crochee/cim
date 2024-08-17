use async_trait::async_trait;
use serde_json::value::RawValue;
use sqlx::{MySqlPool, Row};

use cim_slo::{errors, Result};
use cim_watch::Watcher;

use crate::{
    connector::{Connector, ListParams},
    convert::convert_param,
    Event, Interface, List,
};

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
    type L = ListParams;
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
            row.try_get("resource_version").map_err(errors::any)?;
        output.config = row.try_get("config").map_err(errors::any)?;
        Ok(())
    }
    async fn list(
        &self,
        opts: &Self::L,
        output: &mut List<Self::T>,
    ) -> Result<()> {
        let mut wheres = String::new();
        combine_param(&mut wheres, opts)?;

        if !wheres.is_empty() {
            wheres.push_str(" AND ");
        }
        wheres.push_str(r#"`deleted` = 0"#);
        // 查询total
        if !opts.pagination.count_disable {
            let policy_result = sqlx::query(
                format!(
                    r#"SELECT COUNT(*) as count FROM `connector`
            WHERE {};"#,
                    wheres,
                )
                .as_str(),
            )
            .fetch_one(&self.pool)
            .await
            .map_err(errors::any)?;

            output.total =
                policy_result.try_get("count").map_err(errors::any)?;
        }

        // 查询列表
        opts.pagination.convert(&mut wheres);

        output.limit = opts.pagination.limit;
        output.offset = opts.pagination.offset;

        let rows = sqlx::query(
            format!(
            r#"SELECT `id`,`type`,`name`,`resource_version`,`config`,`connector_data`
            FROM `connector`
            WHERE {};"#,
            wheres
            ).as_str()
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
                    .try_get("resource_version")
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

fn combine_param(wheres: &mut String, opts: &ListParams) -> Result<()> {
    if let Some(v) = &opts.connector_type {
        if !wheres.is_empty() {
            wheres.push_str(" AND ");
        }

        wheres.push_str(r#"`type` = "#);
        convert_param(wheres, v);
    }

    Ok(())
}
