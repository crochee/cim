use async_trait::async_trait;
use chrono::Utc;
use sqlx::{MySqlPool, Row};

use cim_slo::{errors, Result};
use cim_watch::{Watcher, WatcherHub};

use crate::{
    group::{Group, ListParams},
    Event, Interface, List,
};

#[derive(Clone)]
pub struct GroupImpl {
    pool: MySqlPool,
    watch_hub: WatcherHub<Event<Group>>,
}

impl GroupImpl {
    pub fn new(pool: MySqlPool) -> Self {
        Self {
            pool,
            watch_hub: WatcherHub::default(),
        }
    }
}

#[async_trait]
impl Interface for GroupImpl {
    type T = Group;
    type L = ListParams;
    async fn put(&self, input: &Self::T, _ttl: u64) -> Result<()> {
        sqlx::query(
            r#"REPLACE INTO `group`
            (`id`,`account_id`,`name`,`desc`)
            VALUES(?,?,?,?);"#,
        )
        .bind(&input.id)
        .bind(&input.account_id)
        .bind(&input.name)
        .bind(&input.desc)
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
        if sqlx::query(
            r#"SELECT COUNT(*) as count FROM `group_user` WHERE `group_id` = ? AND `deleted` = 0"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(errors::any)?.try_get::<i64,_>("count").map_err(errors::any)?!=0{
            return Err(errors::forbidden(&"can't delete group, because it is used by user".to_string()));
        };
        if sqlx::query(
            r#"SELECT COUNT(*) as count FROM `policy_bindings` WHERE `bindings_type` = 2 AND `bindings_id` = ? AND `deleted` = 0"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(errors::any)?.try_get::<i64,_>("count").map_err(errors::any)?!=0{
            return Err(errors::forbidden(&"can't delete group, because it is attached by policy".to_string()));
        };
        sqlx::query(
            r#"UPDATE `group` SET `deleted` = `id`,`deleted_at`= now()
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
            r#"SELECT `id`,`account_id`,`name`,`desc`,`created_at`,`updated_at`
                FROM `group`
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
        output.account_id = row
            .try_get::<u64, _>("account_id")
            .map_err(errors::any)?
            .to_string();
        output.name = row.try_get("name").map_err(errors::any)?;
        output.desc = row.try_get("desc").map_err(errors::any)?;
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
                    r#"SELECT COUNT(*) as count FROM `group`
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
                r#"SELECT `id`,`account_id`,`name`,`desc`,`created_at`,`updated_at`
                FROM `group`
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
                account_id: row
                    .try_get::<u64, _>("account_id")
                    .map_err(errors::any)?
                    .to_string(),
                name: row.try_get("name").map_err(errors::any)?,
                desc: row.try_get("desc").map_err(errors::any)?,
                created_at: row.try_get("created_at").map_err(errors::any)?,
                updated_at: row.try_get("updated_at").map_err(errors::any)?,
            });
        }

        Ok(())
    }
    fn watch<W: Watcher<Event<Self::T>>>(
        &self,
        handler: W,
        remove: impl Fn() + Send + 'static,
    ) -> Box<dyn Fn() + Send> {
        self.watch_hub
            .watch(Utc::now().timestamp() as usize, handler, remove)
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
                r#"SELECT COUNT(*) as count FROM `group`
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

    if let Some(account_id) = &opts.account_id {
        let account_id_u64: u64 = account_id
            .parse()
            .map_err(|err| errors::bad_request(&err))?;

        if !wheres.is_empty() {
            wheres.push_str(" AND ");
        }
        wheres
            .push_str(format!(r#"`account_id` = {}"#, account_id_u64).as_str());
    }

    Ok(())
}
