use async_trait::async_trait;
use sqlx::{MySqlPool, Row};

use cim_slo::{errors, Result};

use crate::{
    group_user::{GroupUser, ListParams},
    Interface, List,
};

#[derive(Clone, Debug)]
pub struct GroupUserImpl {
    pool: MySqlPool,
}

impl GroupUserImpl {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Interface for GroupUserImpl {
    type T = GroupUser;
    type L = ListParams;

    #[tracing::instrument]
    async fn put(&self, input: &Self::T) -> Result<()> {
        sqlx::query(
            r#"REPLACE INTO `group_user`
            (`id`,`group_id`,`user_id`)
            VALUES(?,?,?);"#,
        )
        .bind(&input.id)
        .bind(&input.group_id)
        .bind(&input.user_id)
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
        sqlx::query(
            r#"UPDATE `group_user` SET `deleted` = `id`,`deleted_at`= now()
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
            r#"SELECT `id`,`group_id`,`user_id`,`created_at`,`updated_at`
                FROM `group_user`
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
        output.group_id = row
            .try_get::<u64, _>("group_id")
            .map_err(errors::any)?
            .to_string();
        output.user_id = row
            .try_get::<u64, _>("user_id")
            .map_err(errors::any)?
            .to_string();
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
                    r#"SELECT COUNT(*) as count FROM `group_user`
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
                r#"SELECT `id`,`group_id`,`user_id`,`created_at`,`updated_at`
                FROM `group_user`
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
                group_id: row
                    .try_get::<u64, _>("group_id")
                    .map_err(errors::any)?
                    .to_string(),
                user_id: row
                    .try_get::<u64, _>("user_id")
                    .map_err(errors::any)?
                    .to_string(),
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
                r#"SELECT COUNT(*) as count FROM `group_user`
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

    if let Some(group_id) = &opts.group_id {
        let group_id_u64: u64 =
            group_id.parse().map_err(|err| errors::bad_request(&err))?;

        if !wheres.is_empty() {
            wheres.push_str(" AND ");
        }
        wheres.push_str(format!(r#"`group_id` = {}"#, group_id_u64).as_str());
    }
    if let Some(user_id) = &opts.user_id {
        let user_id_u64: u64 =
            user_id.parse().map_err(|err| errors::bad_request(&err))?;

        if !wheres.is_empty() {
            wheres.push_str(" AND ");
        }
        wheres.push_str(format!(r#"`user_id` = {}"#, user_id_u64).as_str());
    }

    Ok(())
}
