use async_trait::async_trait;
use chrono::Utc;
use sqlx::{MySqlPool, Row};

use cim_core::{next_id, Error, Result};

use crate::models::{
    policy::{Policy, Statement},
    List, ID,
};

#[derive(Clone)]
pub struct MariadbPolicies {
    pool: MySqlPool,
}

impl MariadbPolicies {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl super::PoliciesRepository for MariadbPolicies {
    async fn create(
        &self,
        id: Option<String>,
        content: &super::Content,
    ) -> Result<ID> {
        let account_id: u64 = content
            .account_id
            .clone()
            .unwrap_or_else(|| "0".to_owned())
            .parse()
            .map_err(|err| Error::BadRequest(format!("{}", err)))?;

        let mut user_id: u64 = 0;
        if account_id > 0 {
            user_id = content
                .user_id
                .clone()
                .unwrap_or_else(|| "0".to_owned())
                .parse()
                .map_err(|err| Error::BadRequest(format!("{}", err)))?;
        };

        let uid = match id {
            Some(v) => v
                .parse()
                .map_err(|err| Error::BadRequest(format!("{}", err)))?,
            None => next_id().map_err(Error::any)?,
        };

        let statement =
            serde_json::to_string(&content.statement).map_err(Error::any)?;

        sqlx::query!(
            r#"INSERT INTO `policy`
            (`id`,`account_id`,`user_id`,`desc`,`version`,`content`)
            VALUES(?,?,?,?,?,?);"#,
            uid,
            account_id,
            user_id,
            content.desc,
            content.version,
            statement,
        )
        .execute(&self.pool)
        .await
        .map_err(Error::any)?;
        Ok(ID {
            id: uid.to_string(),
        })
    }
    async fn update(
        &self,
        id: &str,
        account_id: Option<String>,
        opts: &super::Opts,
    ) -> Result<()> {
        let mut update_content = String::from("");
        if let Some(desc) = &opts.desc {
            update_content.push_str(format!(r#"`desc` = '{}'"#, desc).as_str());
        };
        if let Some(version) = &opts.version {
            if !update_content.is_empty() {
                update_content.push_str(" , ");
            }
            update_content
                .push_str(format!(r#"`version` = '{}'"#, version).as_str());
        };
        if let Some(statement) = &opts.statement {
            if !update_content.is_empty() {
                update_content.push_str(" , ");
            }
            let content =
                serde_json::to_string(statement).map_err(Error::any)?;
            update_content
                .push_str(format!(r#"`content` = '{}'"#, content).as_str());
        };

        if update_content.is_empty() {
            return Ok(());
        }
        let mut wheres = format!(r#"`id` = {}"#, id);
        if let Some(v) = account_id {
            let account_id_u64: u64 = v
                .parse()
                .map_err(|err| Error::BadRequest(format!("{}", err)))?;
            wheres.push_str(" AND ");
            wheres.push_str(
                format!(r#"`account_id` = {}"#, account_id_u64).as_str(),
            );
        }
        if !opts.unscoped.unwrap_or_default() {
            if !wheres.is_empty() {
                wheres.push_str(" AND ");
            }
            wheres.push_str(r#"`deleted` = 0"#);
        } else {
            if !update_content.is_empty() {
                update_content.push_str(" , ");
            }
            update_content.push_str(r#"`deleted` = 0,`deleted_at`=NULL"#);
        };
        sqlx::query(
            format!(
                r#"UPDATE `policy` SET {}
                WHERE {};"#,
                update_content, wheres,
            )
            .as_str(),
        )
        .execute(&self.pool)
        .await
        .map_err(Error::any)?;
        Ok(())
    }
    async fn get(
        &self,
        id: &str,
        account_id: Option<String>,
    ) -> Result<Policy> {
        let mut wheres = format!(r#"`id` = {}"#, id);
        if let Some(v) = account_id {
            let account_id_u64: u64 = v
                .parse()
                .map_err(|err| Error::BadRequest(format!("{}", err)))?;
            wheres.push_str(" AND ");
            wheres.push_str(
                format!(r#"`account_id` = {}"#, account_id_u64).as_str(),
            );
        }

        let row = match sqlx::query(
            format!(r#"SELECT `id`,`account_id`,`user_id`,`desc`,`version`,`content`,`created_at`,`updated_at`
            FROM `policy`
            WHERE {} AND `deleted` = 0;"#,
            wheres)
            .as_str()
        )
        .fetch_optional(&self.pool)
        .await
        {
            Ok(v) => match v {
                Some(value) => Ok(value),
                None => Err(Error::NotFound("no rows".to_owned())),
            },
            Err(err) => Err(Error::any(err)),
        }?;

        let v = row.try_get::<u64, _>("account_id").map_err(Error::any)?;
        let account_id_str = if v == 0 { None } else { Some(v.to_string()) };
        let v = row.try_get::<u64, _>("user_id").map_err(Error::any)?;
        let user_id_str = if v == 0 { None } else { Some(v.to_string()) };
        let v = row.try_get::<String, _>("content").map_err(Error::any)?;
        let statement: Vec<Statement> =
            serde_json::from_str(&v).map_err(Error::any)?;
        Ok(Policy {
            id: row.try_get::<u64, _>("id").map_err(Error::any)?.to_string(),
            account_id: account_id_str,
            user_id: user_id_str,
            desc: row.try_get("desc").map_err(Error::any)?,
            version: row.try_get("version").map_err(Error::any)?,
            statement,
        })
    }

    async fn delete(&self, id: &str, account_id: Option<String>) -> Result<()> {
        let mut wheres = format!(r#"`id` = {}"#, id);
        if let Some(v) = account_id {
            let account_id_u64: u64 = v
                .parse()
                .map_err(|err| Error::BadRequest(format!("{}", err)))?;
            wheres.push_str(" AND ");
            wheres.push_str(
                format!(r#"`account_id` = {}"#, account_id_u64).as_str(),
            );
        }
        sqlx::query(
            format!(
                r#"UPDATE `policy` SET `deleted` = `id`,`deleted_at`= '{}'
            WHERE {} AND `deleted` = 0;"#,
                Utc::now().naive_utc(),
                wheres
            )
            .as_str(),
        )
        .execute(&self.pool)
        .await
        .map_err(Error::any)?;
        Ok(())
    }

    async fn list(&self, filter: &super::Querys) -> Result<List<Policy>> {
        let mut wheres = String::from("");
        if let Some(version) = &filter.version {
            wheres.push_str(format!(r#"`version` = {}"#, version).as_str());
        };
        if let Some(account_id) = &filter.account_id {
            let account_id_u64: u64 = account_id
                .parse()
                .map_err(|err| Error::BadRequest(format!("{}", err)))?;
            if !wheres.is_empty() {
                wheres.push_str(" AND ");
            }
            wheres.push_str(
                format!(r#"`account_id` IN (0,{})"#, account_id_u64).as_str(),
            );
        };
        if !wheres.is_empty() {
            wheres.push_str(" AND ");
        }
        wheres.push_str(r#"`deleted` = 0"#);
        // 查询total
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
        .map_err(Error::any)?;
        // 查询列表
        let query = filter.pagination.to_string();
        if !query.is_empty() {
            wheres.push_str(query.as_str());
        }
        let rows = sqlx::query(
            format!(
                r#"SELECT `id`,`account_id`,`user_id`,`desc`,`version`,`content`,`created_at`,`updated_at`
                FROM `policy`
                WHERE {};"#,
                wheres,
            )
            .as_str(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::any)?;
        let mut result = List {
            data: Vec::new(),
            limit: filter.pagination.limit,
            offset: filter.pagination.offset,
            total: policy_result.try_get("count").map_err(Error::any)?,
        };
        for row in rows.iter() {
            let v = row.try_get::<u64, _>("account_id").map_err(Error::any)?;
            let account_id_str =
                if v == 0 { None } else { Some(v.to_string()) };
            let v = row.try_get::<u64, _>("user_id").map_err(Error::any)?;
            let user_id_str = if v == 0 { None } else { Some(v.to_string()) };
            let v = row.try_get::<String, _>("content").map_err(Error::any)?;
            let statement: Vec<Statement> =
                serde_json::from_str(&v).map_err(Error::any)?;

            result.data.push(Policy {
                id: row
                    .try_get::<u64, _>("id")
                    .map_err(Error::any)?
                    .to_string(),
                account_id: account_id_str,
                user_id: user_id_str,
                desc: row.try_get("desc").map_err(Error::any)?,
                version: row.try_get("version").map_err(Error::any)?,
                statement,
            })
        }
        Ok(result)
    }
    async fn exist(
        &self,
        id: &str,
        account_id: Option<String>,
        unscoped: bool,
    ) -> Result<bool> {
        let mut wheres = format!(r#"`id` = {}"#, id);
        if let Some(v) = account_id {
            let account_id_u64: u64 = v
                .parse()
                .map_err(|err| Error::BadRequest(format!("{}", err)))?;
            wheres.push_str(" AND ");
            wheres.push_str(
                format!(r#"`account_id` = {}"#, account_id_u64).as_str(),
            );
        }
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
        .map_err(Error::any)?;
        let count: i64 = result.try_get("count").map_err(Error::any)?;
        Ok(count != 0)
    }
}
