use async_trait::async_trait;
use chrono::Utc;
use sqlx::{MySqlPool, Row};

use cim_slo::{errors, Result};
use cim_watch::{WatchGuard, Watcher, WatcherHub};

use crate::{
    convert::convert_param,
    policy_binding::{ListParams, PolicyBinding},
    Event, Interface, List,
};

#[derive(Clone)]
pub struct PolicyBindingImpl {
    pool: MySqlPool,
    watch_hub: WatcherHub<Event<PolicyBinding>>,
}

impl PolicyBindingImpl {
    pub fn new(pool: MySqlPool) -> Self {
        Self {
            pool,
            watch_hub: WatcherHub::default(),
        }
    }
}

#[async_trait]
impl Interface for PolicyBindingImpl {
    type T = PolicyBinding;
    type L = ListParams;
    async fn put(&self, input: &Self::T, _ttl: u64) -> Result<()> {
        let binding_type: u8 = (&input.bindings_type).into();
        sqlx::query(
            r#"REPLACE INTO `policy_binding`
            (`id`,`policy_id`,`bindings_type`,`bindings_id`)
            VALUES(?,?,?,?);"#,
        )
        .bind(&input.id)
        .bind(&input.policy_id)
        .bind(binding_type)
        .bind(&input.bindings_id)
        .execute(&self.pool)
        .await
        .map_err(errors::any)?;
        self.watch_hub.notify(
            Utc::now().timestamp() as usize,
            Event::Put(input.to_owned()),
        );
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<()> {
        let id = id.parse::<u64>().map_err(|err| errors::bad_request(&err))?;
        sqlx::query(
            r#"UPDATE `policy_binding` SET `deleted` = `id`,`deleted_at`= now()
            WHERE id = ? AND `deleted` = 0;"#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(errors::any)?;
        self.watch_hub.notify(
            Utc::now().timestamp() as usize,
            Event::Delete(Self::T {
                id: id.to_string(),
                ..Default::default()
            }),
        );
        Ok(())
    }
    async fn get(&self, id: &str, output: &mut Self::T) -> Result<()> {
        let id = id.parse::<u64>().map_err(|err| errors::bad_request(&err))?;
        let row = match sqlx::query(
            r#"SELECT `id`,`policy_id`,`bindings_type`,`bindings_id`,`created_at`,`updated_at`
                FROM `policy_binding`
                WHERE id = ? AND `deleted` = 0;"#,
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
        output.policy_id = row
            .try_get::<u64, _>("policy_id")
            .map_err(errors::any)?
            .to_string();
        output.bindings_type = row
            .try_get::<u8, _>("bindings_type")
            .map_err(errors::any)?
            .into();
        output.bindings_id = row.try_get("bindings_id").map_err(errors::any)?;
        output.created_at = row.try_get("created_at").map_err(errors::any)?;
        output.updated_at = row.try_get("updated_at").map_err(errors::any)?;
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
                    r#"SELECT COUNT(*) as count FROM `policy_binding`
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
                r#"SELECT `id`,`policy_id`,`bindings_type`,`bindings_id`,`created_at`,`updated_at`
                FROM `policy_binding`
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
                policy_id: row
                    .try_get::<u64, _>("policy_id")
                    .map_err(errors::any)?
                    .to_string(),
                bindings_type: row
                    .try_get::<u8, _>("bindings_type")
                    .map_err(errors::any)?
                    .into(),
                bindings_id: row.try_get("bindings_id").map_err(errors::any)?,
                created_at: row.try_get("created_at").map_err(errors::any)?,
                updated_at: row.try_get("updated_at").map_err(errors::any)?,
            });
        }

        Ok(())
    }
    fn watch<W: Watcher<Event<Self::T>>>(
        &self,
        handler: W,
    ) -> Box<dyn WatchGuard + Send> {
        self.watch_hub
            .watch(Utc::now().timestamp() as usize, handler)
    }
    async fn count(&self, opts: &Self::L, unscoped: bool) -> Result<i64> {
        let mut wheres = String::new();
        combine_param(&mut wheres, opts)?;
        if !unscoped {
            wheres.push_str(" AND ");
            wheres.push_str(r#"`deleted` = 0"#);
        }
        let result = sqlx::query(
            format!(
                r#"SELECT COUNT(*) as count FROM `policy_binding`
            WHERE {} LIMIT 1;"#,
                wheres,
            )
            .as_str(),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(errors::any)?;
        result.try_get("count").map_err(errors::any)
    }
}

fn combine_param(wheres: &mut String, opts: &ListParams) -> Result<()> {
    if let Some(v) = &opts.id {
        if !wheres.is_empty() {
            wheres.push_str(" AND ");
        }

        wheres.push_str(
            format!(
                r#"`id` = {}"#,
                v.parse::<u64>().map_err(|err| errors::bad_request(&err))?
            )
            .as_str(),
        );
    }

    if let Some(policy_id) = &opts.policy_id {
        let policy_id_u64: u64 =
            policy_id.parse().map_err(|err| errors::bad_request(&err))?;

        if !wheres.is_empty() {
            wheres.push_str(" AND ");
        }
        wheres.push_str(format!(r#"`policy_id` = {}"#, policy_id_u64).as_str());
    }
    if let Some(bindings_type) = &opts.bindings_type {
        let binding_type_u8: u8 = bindings_type.into();
        if !wheres.is_empty() {
            wheres.push_str(" AND ");
        }
        wheres.push_str(
            format!(r#"`bindings_type` = {}"#, binding_type_u8).as_str(),
        );
    }
    if let Some(binding_id) = &opts.bindings_id {
        if !wheres.is_empty() {
            wheres.push_str(" AND ");
        }
        wheres.push_str(r#"`bindings_id` = "#);
        convert_param(wheres, binding_id);
    }
    Ok(())
}
