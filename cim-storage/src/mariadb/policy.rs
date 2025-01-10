use async_trait::async_trait;
use sqlx::{types::Json, MySqlPool, Row};

use cim_pim::{Request, Statement};
use cim_slo::{errors, Result};

use crate::{
    convert::convert_param,
    policy::{ListParams, Policy, StatementStore},
    Interface, List,
};

#[derive(Clone, Debug)]
pub struct PolicyImpl {
    pool: MySqlPool,
}

impl PolicyImpl {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Interface for PolicyImpl {
    type T = Policy;
    type L = ListParams;

    #[tracing::instrument]
    async fn put(&self, input: &Self::T) -> Result<()> {
        sqlx::query(
            r#"REPLACE INTO `policy`
            (`id`,`account_id`,`desc`,`version`,`statement`)
            VALUES(?,?,?,?,?);"#,
        )
        .bind(&input.id)
        .bind(&input.account_id)
        .bind(&input.desc)
        .bind(&input.version)
        .bind(Json(&input.statement))
        .execute(&self.pool)
        .await
        .map_err(errors::any)?;

        Ok(())
    }

    #[tracing::instrument]
    async fn delete(&self, input: &Self::T) -> Result<()> {
        let id = input
            .id
            .parse::<u64>()
            .map_err(|err| errors::bad_request(&err))?;
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

        Ok(())
    }

    #[tracing::instrument]
    async fn get(&self, output: &mut Self::T) -> Result<()> {
        let id = output
            .id
            .parse::<u64>()
            .map_err(|err| errors::bad_request(&err))?;
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
        output.account_id = row
            .try_get::<Option<u64>, _>("account_id")
            .map_err(errors::any)?
            .map(|v| v.to_string());
        output.desc = row.try_get("desc").map_err(errors::any)?;
        output.version = row.try_get("version").map_err(errors::any)?;
        output.statement = row
            .try_get::<Json<Vec<Statement>>, _>("statement")
            .map_err(errors::any)?
            .0;
        output.created_at = row.try_get("created_at").map_err(errors::any)?;
        output.updated_at = row.try_get("updated_at").map_err(errors::any)?;
        Ok(())
    }

    #[tracing::instrument]
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
                FROM `policy`
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
                    .try_get::<Option<u64>, _>("account_id")
                    .map_err(errors::any)?
                    .map(|v| v.to_string()),
                desc: row.try_get("desc").map_err(errors::any)?,
                version: row.try_get("version").map_err(errors::any)?,
                statement: row
                    .try_get::<Json<Vec<Statement>>, _>("statement")
                    .map_err(errors::any)?
                    .0,
                created_at: row.try_get("created_at").map_err(errors::any)?,
                updated_at: row.try_get("updated_at").map_err(errors::any)?,
            });
        }

        Ok(())
    }

    #[tracing::instrument]
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
        let rows = sqlx::query(r#"SELECT t2.`statement`
            FROM (
                (
                SELECT `policy_id` FROM `policy_binding` WHERE `bindings_id` = ? AND `bindings_type` = 1 AND `deleted` = 0
                )
                UNION
                (
                SELECT a3.`policy_id` FROM `group_user` a1 RIGHT JOIN `group` a2 ON a1.`group_id` = a2.`id`
                RIGHT JOIN `policy_binding` a3 ON a2.`id` = a3.`bindings_id`
                WHERE a1.`user_id` = ? AND a1.`deleted` = 0 AND
                a2.`deleted` = 0 AND a3.`bindings_type` = 2 AND a3.`deleted` = 0
                )
                UNION
                (
                SELECT b3.`policy_id` FROM `role_binding` b1 RIGHT JOIN `role` b2 ON b1.`role_id` = b2.`id`
                RIGHT JOIN `policy_binding` b3 ON b2.`id` = b3.`bindings_id`
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
            let v =
                row.try_get::<String, _>("statement").map_err(errors::any)?;
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
