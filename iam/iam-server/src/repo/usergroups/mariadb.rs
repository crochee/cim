use async_trait::async_trait;
use chrono::Utc;
use sqlx::{MySqlPool, Row};

use cim_core::{next_id, Error, Result};

use crate::models::{List, ID};

#[derive(Clone)]
pub struct MariadbUserGroups {
    pool: MySqlPool,
}

impl MariadbUserGroups {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl super::UserGroupsRepository for MariadbUserGroups {
    async fn create(
        &self,
        id: Option<String>,
        content: &super::Content,
    ) -> Result<ID> {
        let uid = match id {
            Some(v) => v
                .parse()
                .map_err(|err| Error::BadRequest(format!("{}", err)))?,
            None => next_id().map_err(Error::any)?,
        };
        let account_id: u64 = content
            .account_id
            .parse()
            .map_err(|err| Error::BadRequest(format!("{}", err)))?;
        let user_id: u64 = content
            .user_id
            .parse()
            .map_err(|err| Error::BadRequest(format!("{}", err)))?;
        sqlx::query!(
            r#"INSERT INTO `user_group`
            (`id`,`account_id`,`user_id`,`name`,`desc`)
            VALUES(?,?,?,?,?);"#,
            uid,
            account_id,
            user_id,
            content.name,
            content.desc,
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
        if let Some(name) = &opts.name {
            update_content.push_str(format!(r#"`name` = '{}'"#, name).as_str());
        };
        if let Some(desc) = &opts.desc {
            if !update_content.is_empty() {
                update_content.push_str(" , ");
            }
            update_content.push_str(format!(r#"`desc` = '{}'"#, desc).as_str());
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
            update_content.push_str(r#"`deleted` = 0 , `deleted_at` = NULL"#);
        };
        sqlx::query(
            format!(
                r#"UPDATE `user_group` SET {}
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
    ) -> Result<super::UserGroup> {
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
            format!(r#"SELECT `id`,`account_id`,`user_id`,`name`,`desc`,`created_at`,`updated_at`
            FROM `user_group`
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
        Ok(super::UserGroup {
            id: row.try_get::<u64, _>("id").map_err(Error::any)?.to_string(),
            account_id: row
                .try_get::<u64, _>("account_id")
                .map_err(Error::any)?
                .to_string(),
            user_id: row
                .try_get::<u64, _>("user_id")
                .map_err(Error::any)?
                .to_string(),
            name: row.try_get("name").map_err(Error::any)?,
            desc: row.try_get("desc").map_err(Error::any)?,
            created_at: row.try_get("created_at").map_err(Error::any)?,
            updated_at: row.try_get("updated_at").map_err(Error::any)?,
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
                r#"UPDATE `user_group` SET `deleted` = `id`,`deleted_at`= '{}'
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

    async fn list(
        &self,
        filter: &super::Querys,
    ) -> Result<List<super::UserGroup>> {
        let mut wheres = String::from("");
        if let Some(account_id) = &filter.account_id {
            let account_id_u64: u64 = account_id
                .parse()
                .map_err(|err| Error::BadRequest(format!("{}", err)))?;
            if !wheres.is_empty() {
                wheres.push_str(" AND ");
            }
            wheres.push_str(
                format!(r#"`account_id` = {}"#, account_id_u64).as_str(),
            );
        };
        if !wheres.is_empty() {
            wheres.push_str(" AND ");
        }
        wheres.push_str(r#"`deleted` = 0"#);
        // 查询total
        let policy_result = sqlx::query(
            format!(
                r#"SELECT COUNT(*) as count FROM `user_group`
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
                r#"SELECT `id`,`account_id`,`user_id`,`name`,`desc`,`created_at`,`updated_at`
                FROM `user_group`
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
            result.data.push(super::UserGroup {
                id: row
                    .try_get::<u64, _>("id")
                    .map_err(Error::any)?
                    .to_string(),
                account_id: row
                    .try_get::<u64, _>("account_id")
                    .map_err(Error::any)?
                    .to_string(),
                user_id: row
                    .try_get::<u64, _>("user_id")
                    .map_err(Error::any)?
                    .to_string(),
                name: row.try_get("name").map_err(Error::any)?,
                desc: row.try_get("desc").map_err(Error::any)?,
                created_at: row.try_get("created_at").map_err(Error::any)?,
                updated_at: row.try_get("updated_at").map_err(Error::any)?,
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
                r#"SELECT COUNT(*) as count FROM `user_group`
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
