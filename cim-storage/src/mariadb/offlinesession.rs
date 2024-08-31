use std::collections::HashMap;

use async_trait::async_trait;
use serde_json::value::RawValue;
use sqlx::{types::Json, MySqlPool, Row};

use cim_slo::{errors, Result};
use cim_watch::{WatchGuard, Watcher};

use crate::{
    convert::convert_param,
    offlinesession::{ListParams, OfflineSession, RefreshTokenRef},
    Event, Interface, List,
};

#[derive(Clone)]
pub struct OfflineSessionImpl {
    pool: MySqlPool,
}

impl OfflineSessionImpl {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Interface for OfflineSessionImpl {
    type T = OfflineSession;
    type L = ListParams;
    async fn put(&self, content: &Self::T, _ttl: u64) -> Result<()> {
        sqlx::query(
            r#"REPLACE INTO `offline_session`
            (`id`,`user_id`,`conn_id`,`refresh`,`connector_data`)
            VALUES(?,?,?,?,?);"#,
        )
        .bind(&content.id)
        .bind(&content.user_id)
        .bind(&content.conn_id)
        .bind(Json(&content.refresh))
        .bind(content.connector_data.as_ref().map(|v| v.to_string()))
        .execute(&self.pool)
        .await
        .map_err(errors::any)?;
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<()> {
        sqlx::query(
            r#"UPDATE `offline_session` SET `deleted` = `id`,`deleted_at`= now()
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
            r#"SELECT `id`,`user_id`,`conn_id`,`refresh`,`connector_data`
             FROM `offline_session`
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
        output.refresh = row
            .try_get::<Json<HashMap<String, RefreshTokenRef>>, _>("refresh")
            .map_err(errors::any)?
            .0;
        output.connector_data = row
            .try_get::<Option<String>, _>("connector_data")
            .map_err(errors::any)?
            .map(|v| RawValue::from_string(v).unwrap());
        output.user_id =
            row.try_get::<String, _>("user_id").map_err(errors::any)?;
        output.conn_id =
            row.try_get::<String, _>("conn_id").map_err(errors::any)?;
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
                    r#"SELECT COUNT(*) as count FROM `offline_session`
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
                r#"SELECT `id`,`user_id`,`conn_id`,`refresh`,`connector_data`
            FROM `offline_session`
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
                user_id: row
                    .try_get::<String, _>("user_id")
                    .map_err(errors::any)?,
                conn_id: row
                    .try_get::<String, _>("conn_id")
                    .map_err(errors::any)?,
                refresh: row
                    .try_get::<Json<HashMap<String, RefreshTokenRef>>, _>(
                        "refresh",
                    )
                    .map_err(errors::any)?
                    .0,
                connector_data: row
                    .try_get::<Option<String>, _>("connector_data")
                    .map_err(errors::any)?
                    .map(|v| RawValue::from_string(v).unwrap()),
            })
        }
        Ok(())
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

fn combine_param(wheres: &mut String, opts: &ListParams) -> Result<()> {
    if let Some(user_id) = &opts.user_id {
        if !wheres.is_empty() {
            wheres.push_str(" AND ");
        }
        wheres.push_str(r#"`user_id` = "#);
        convert_param(wheres, user_id);
    }
    if let Some(conn_id) = &opts.conn_id {
        if !wheres.is_empty() {
            wheres.push_str(" AND ");
        }
        wheres.push_str(r#"`conn_id` = "#);
        convert_param(wheres, conn_id);
    }
    Ok(())
}
