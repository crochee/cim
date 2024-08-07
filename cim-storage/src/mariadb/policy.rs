use async_trait::async_trait;
use chrono::Utc;
use cim_pim::{Request, Statement};
use sqlx::{MySqlPool, Row};

use cim_slo::{errors, Result};
use cim_watch::{Watcher, WatcherHub};

use crate::{
    convert::convert_param,
    policy::{ListParams, Policy, StatementStore},
    Event, Interface, List,
};

#[derive(Clone)]
pub struct PolicyImpl {
    pool: MySqlPool,
    watch_hub: WatcherHub<Event<Policy>>,
}

impl PolicyImpl {
    pub fn new(pool: MySqlPool) -> Self {
        Self {
            pool,
            watch_hub: WatcherHub::default(),
        }
    }
}

#[async_trait]
impl Interface for PolicyImpl {
    type T = Policy;
    type L = ListParams;
    async fn put(&self, input: &Self::T, _ttl: u64) -> Result<()> {
        let statement =
            serde_json::to_string(&input.statement).map_err(errors::any)?;
        sqlx::query(
            r#"REPLACE INTO `policy`
            (`id`,`account_id`,`desc`,`version`,`statement`)
            VALUES(?,?,?,?,?);"#,
        )
        .bind(&input.id)
        .bind(&input.account_id)
        .bind(&input.desc)
        .bind(&input.version)
        .bind(statement)
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
            r#"SELECT COUNT(*) as count FROM `policy_bindings` WHERE `policy_id` = ? AND `deleted` = 0"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(errors::any)?.try_get::<i64,_>("count").map_err(errors::any)?!=0{
            return Err(errors::forbidden(&"can't delete policy, because it is used".to_string()));
        };
        sqlx::query(
            r#"UPDATE `policy` SET `deleted` = `id`,`deleted_at`= now()
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
            r#"SELECT `id`,`account_id`,`desc`,`version`,`statement`,`created_at`,`updated_at`
                FROM `policy`
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
        output.account_id = row.try_get("account_id").map_err(errors::any)?;
        output.desc = row.try_get("desc").map_err(errors::any)?;
        output.version = row.try_get("version").map_err(errors::any)?;

        output.statement = serde_json::from_str(
            &row.try_get::<String, _>("statement").map_err(errors::any)?,
        )
        .map_err(errors::any)?;

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
                    r#"SELECT COUNT(*) as count FROM `policy`
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
                r#"SELECT `id`,`account_id`,`desc`,`version`,`statement`,`created_at`,`updated_at`
                FROM `role`
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
                account_id: row.try_get("account_id").map_err(errors::any)?,
                desc: row.try_get("desc").map_err(errors::any)?,
                version: row.try_get("version").map_err(errors::any)?,
                statement: serde_json::from_str(
                    &row.try_get::<String, _>("statement")
                        .map_err(errors::any)?,
                )
                .map_err(errors::any)?,
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
                r#"SELECT COUNT(*) as count FROM `policy`
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

    if let Some(version) = &opts.version {
        if !wheres.is_empty() {
            wheres.push_str(" AND ");
        }
        wheres.push_str(r#"`version` = "#);
        convert_param(wheres, version);
    }
    if let Some(account_id) = &opts.account_id {
        let account_id_u64: u64 = account_id
            .parse()
            .map_err(|err| errors::bad_request(&err))?;

        wheres
            .push_str(format!(r#"`account_id` = {}"#, account_id_u64).as_str());
    }
    if let Some(group_id) = &opts.group_id {
        if !wheres.is_empty() {
            wheres.push_str(" AND ");
        }
        let group_id_u64: u64 =
            group_id.parse().map_err(|err| errors::bad_request(&err))?;
        wheres.push_str(format!(r#"`id` IN (SELECT `policy_id` FROM `policy_bindings` WHERE `bindings_id` = {} AND `bindings_type` = 2 AND `deleted` = 0)"#,
            group_id_u64).as_str());
    };
    if let Some(user_id) = &opts.user_id {
        if !wheres.is_empty() {
            wheres.push_str(" AND ");
        }
        let user_id_u64: u64 =
            user_id.parse().map_err(|err| errors::bad_request(&err))?;
        wheres.push_str(format!(r#"`id` IN (SELECT `bindings_id` FROM `policy_bindings` WHERE `bingdings_id` = {} AND `bindings_type` = 1 AND `deleted` = 0)"#,
            user_id_u64).as_str());
    };
    if let Some(role_id) = &opts.role_id {
        if !wheres.is_empty() {
            wheres.push_str(" AND ");
        }
        let role_id_u64: u64 =
            role_id.parse().map_err(|err| errors::bad_request(&err))?;
        wheres.push_str(format!(r#"`id` IN (SELECT `bindings_id` FROM `policy_bindings` WHERE `bingdings_id` = {} AND `bindings_type` = 3 AND `deleted` = 0)"#,
            role_id_u64).as_str());
    }
    Ok(())
}

#[async_trait]
impl StatementStore for PolicyImpl {
    async fn get_statement(&self, req: &Request) -> Result<Vec<Statement>> {
        let user_id = req.subject.parse::<u64>().map_err(errors::any)?;
        let rows = sqlx::query(r#"SELECT t2.`content`
            FROM (
                (
                SELECT `policy_id` FROM `policy_bindings` WHERE `bindings_id` = ? AND `bindings_type` = 1 AND `deleted` = 0
                )
                UNION
                (
                SELECT a3.`policy_id` FROM `group_user` a1 RIGHT JOIN `group` a2 ON a1.`group_id` = a2.`id`
                RIGHT JOIN `policy_bindings` a3 ON a2.`id` = a3.`bindings_id`
                WHERE a1.`user_id` = ? AND a1.`deleted` = 0 AND
                a2.`deleted` = 0 AND a3.`bindings_type` = 2 AND a3.`deleted` = 0
                )
                UNION
                (
                SELECT b3.`policy_id` FROM `role_bindings` b1 RIGHT JOIN `role` b2 ON b1.`role_id` = b2.`id`
                RIGHT JOIN `policy_bindings` b3 ON b2.`id` = b3.`bindings_id`
                WHERE b1.`user_id` = ? AND b1.`deleted` = 0 AND
                b2.`deleted` = 0 AND b3.`bindings_type` = 3 AND b3.`deleted` = 0
                )
            )
            t1 RIGHT JOIN `policy` t2 ON t1.`policy_id`=t2.`id` WHERE t2.`deleted`=0;"#)
            .bind(user_id)
            .bind(user_id)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .map_err(errors::any)?;

        let mut result = Vec::with_capacity(rows.len());

        for row in rows.iter() {
            let v = row.try_get::<String, _>("content").map_err(errors::any)?;
            let mut statement: Vec<Statement> =
                serde_json::from_str(&v).map_err(errors::any)?;
            result.append(&mut statement);
        }
        Ok(result)
    }
}

#[async_trait]
impl<T: StatementStore + Sync> StatementStore for Vec<T> {
    async fn get_statement(&self, req: &Request) -> Result<Vec<Statement>> {
        let mut result = Vec::with_capacity(self.len());
        for store in self.iter() {
            result.append(&mut store.get_statement(req).await?);
        }
        Ok(result)
    }
}
