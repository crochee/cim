use async_trait::async_trait;
use cim_pim::{Request, Statement};
use sqlx::{MySqlPool, Row};

use cim_slo::{errors, next_id, Result};

use crate::{
    convert::{convert_param, update_set_param},
    List, ID,
};

use super::{
    BindingsType, Content, ListOpts, Policy, PolicyStore, StatementStore,
    UpdateOpts,
};

pub struct PolicyImpl {
    pool: MySqlPool,
}

impl PolicyImpl {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PolicyStore for PolicyImpl {
    async fn create_policy(
        &self,
        id: Option<String>,
        content: &Content,
    ) -> Result<ID> {
        let account_id: u64 = content
            .account_id
            .clone()
            .unwrap_or_else(|| "0".to_owned())
            .parse()
            .map_err(|err| errors::bad_request(&err))?;

        let uid = match id {
            Some(v) => v.parse().map_err(|err| errors::bad_request(&err))?,
            None => next_id().map_err(errors::any)?,
        };

        let statement =
            serde_json::to_string(&content.statement).map_err(errors::any)?;

        sqlx::query(
            r#"INSERT INTO `policy`
            (`id`,`account_id`,`desc`,`version`,`content`)
            VALUES(?,?,?,?,?);"#,
        )
        .bind(uid)
        .bind(account_id)
        .bind(&content.desc)
        .bind(&content.version)
        .bind(statement)
        .execute(&self.pool)
        .await
        .map_err(errors::any)?;
        Ok(ID {
            id: uid.to_string(),
        })
    }

    async fn update_policy(
        &self,
        id: &str,
        account_id: Option<String>,
        opts: &UpdateOpts,
    ) -> Result<()> {
        let mut update_content = String::new();

        update_set_param(&mut update_content, r#"`desc` = "#, &opts.desc);

        update_set_param(&mut update_content, r#"`version` = "#, &opts.version);

        let mut statement_content = None;
        if let Some(statement) = &opts.statement {
            if !update_content.is_empty() {
                update_content.push_str(" , ");
            }
            let content =
                serde_json::to_string(statement).map_err(errors::any)?;

            update_content.push_str(r#"`content` = ? "#);
            statement_content = Some(content);
        };

        if update_content.is_empty() {
            return Ok(());
        }
        let mut wheres = format!(
            r#"`id` = {}"#,
            id.parse::<u64>()
                .map_err(|err| { errors::bad_request(&err) })?
        );
        if let Some(v) = account_id {
            let account_id_u64: u64 =
                v.parse().map_err(|err| errors::bad_request(&err))?;
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
        let sqlx_query_sts = format!(
            r#"UPDATE `policy` SET {}
                WHERE {};"#,
            update_content, wheres,
        );
        let mut sqlx_query = sqlx::query(sqlx_query_sts.as_str());
        if let Some(v) = statement_content {
            sqlx_query = sqlx_query.bind(v);
        }
        sqlx_query.execute(&self.pool).await.map_err(errors::any)?;
        Ok(())
    }

    async fn get_policy(
        &self,
        id: &str,
        account_id: Option<String>,
    ) -> Result<Policy> {
        let mut wheres = format!(
            r#"`id` = {}"#,
            id.parse::<u64>().map_err(|err| errors::bad_request(&err))?
        );
        if let Some(v) = account_id {
            let account_id_u64: u64 =
                v.parse().map_err(|err| errors::bad_request(&err))?;
            wheres.push_str(" AND ");
            wheres.push_str(
                format!(r#"`account_id` = {}"#, account_id_u64).as_str(),
            );
        }

        let row = match sqlx::query(
            format!(r#"SELECT `id`,`account_id`,`desc`,`version`,`content`,`created_at`,`updated_at`
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
                None => Err(errors::not_found("no rows")),
            },
            Err(err) => Err(errors::any(err)),
        }?;

        let v = row.try_get::<u64, _>("account_id").map_err(errors::any)?;
        let account_id_str = if v == 0 { None } else { Some(v.to_string()) };
        let v = row.try_get::<String, _>("content").map_err(errors::any)?;
        tracing::debug!("content: {}", v);
        let statement: Vec<Statement> =
            serde_json::from_str(&v).map_err(errors::any)?;
        Ok(Policy {
            id: row
                .try_get::<u64, _>("id")
                .map_err(errors::any)?
                .to_string(),
            account_id: account_id_str,
            desc: row.try_get("desc").map_err(errors::any)?,
            version: row.try_get("version").map_err(errors::any)?,
            statement,
            created_at: row.try_get("created_at").map_err(errors::any)?,
            updated_at: row.try_get("updated_at").map_err(errors::any)?,
        })
    }
    async fn delete_policy(
        &self,
        id: &str,
        account_id: Option<String>,
    ) -> Result<()> {
        let mut wheres = format!(
            r#"`id` = {}"#,
            id.parse::<u64>().map_err(|err| errors::bad_request(&err))?
        );
        if let Some(v) = account_id {
            let account_id_u64: u64 =
                v.parse().map_err(|err| errors::bad_request(&err))?;
            wheres.push_str(" AND ");
            wheres.push_str(
                format!(r#"`account_id` = {}"#, account_id_u64).as_str(),
            );
        }
        if sqlx::query(
            r#"SELECT COUNT(*) as count FROM `policy_bindings` WHERE `policy_id` = ? AND `deleted` = 0"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(errors::any)?.try_get::<i64,_>("count").map_err(errors::any)?!=0{
            return Err(errors::forbidden(&"can't delete policy, because it is used".to_string()));
        }
        sqlx::query(
            format!(
                r#"UPDATE `policy` SET `deleted` = `id`,`deleted_at`= now()
                WHERE {} AND `deleted` = 0;"#,
                wheres
            )
            .as_str(),
        )
        .execute(&self.pool)
        .await
        .map_err(errors::any)?;
        Ok(())
    }
    async fn list_policy(&self, filter: &ListOpts) -> Result<List<Policy>> {
        let mut wheres = String::from("");
        if let Some(version) = &filter.version {
            wheres.push_str(r#"`version` = "#);
            convert_param(&mut wheres, version);
        };
        if let Some(account_id) = &filter.account_id {
            let account_id_u64: u64 = account_id
                .parse()
                .map_err(|err| errors::bad_request(&err))?;
            if !wheres.is_empty() {
                wheres.push_str(" AND ");
            }
            // 0标识公共策略由系统提供
            wheres.push_str(
                format!(r#"`account_id` IN (0,{})"#, account_id_u64).as_str(),
            );
        };

        if let Some(group_id) = &filter.group_id {
            if !wheres.is_empty() {
                wheres.push_str(" AND ");
            }
            let group_id_u64: u64 =
                group_id.parse().map_err(|err| errors::bad_request(&err))?;
            wheres.push_str(format!(r#"`id` IN (SELECT `bindings_id` FROM `policy_bindings` WHERE `bingdings_id` = {} AND `bindings_type` = 2 AND `deleted` = 0)"#,
            group_id_u64).as_str());
        };
        if let Some(user_id) = &filter.user_id {
            if !wheres.is_empty() {
                wheres.push_str(" AND ");
            }
            let user_id_u64: u64 =
                user_id.parse().map_err(|err| errors::bad_request(&err))?;
            wheres.push_str(format!(r#"`id` IN (SELECT `bindings_id` FROM `policy_bindings` WHERE `bingdings_id` = {} AND `bindings_type` = 1 AND `deleted` = 0)"#,
            user_id_u64).as_str());
        };
        if let Some(role_id) = &filter.role_id {
            if !wheres.is_empty() {
                wheres.push_str(" AND ");
            }
            let role_id_u64: u64 =
                role_id.parse().map_err(|err| errors::bad_request(&err))?;
            wheres.push_str(format!(r#"`id` IN (SELECT `bindings_id` FROM `policy_bindings` WHERE `bingdings_id` = {} AND `bindings_type` = 3 AND `deleted` = 0)"#,
            role_id_u64).as_str());
        }
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
        .map_err(errors::any)?;
        // 查询列表
        filter.pagination.convert(&mut wheres);
        let rows = sqlx::query(
            format!(
                r#"SELECT `id`,`account_id`,`desc`,`version`,`content`,`created_at`,`updated_at`
                FROM `policy`
                WHERE {};"#,
                wheres,
            )
            .as_str(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(errors::any)?;
        let mut result = List {
            data: Vec::with_capacity(rows.len()),
            limit: filter.pagination.limit,
            offset: filter.pagination.offset,
            total: policy_result.try_get("count").map_err(errors::any)?,
        };
        for row in rows.iter() {
            let v = row.try_get::<u64, _>("account_id").map_err(errors::any)?;
            let account_id_str =
                if v == 0 { None } else { Some(v.to_string()) };
            let v = row.try_get::<String, _>("content").map_err(errors::any)?;
            let statement: Vec<Statement> =
                serde_json::from_str(&v).map_err(errors::any)?;

            result.data.push(Policy {
                id: row
                    .try_get::<u64, _>("id")
                    .map_err(errors::any)?
                    .to_string(),
                account_id: account_id_str,
                desc: row.try_get("desc").map_err(errors::any)?,
                version: row.try_get("version").map_err(errors::any)?,
                statement,
                created_at: row.try_get("created_at").map_err(errors::any)?,
                updated_at: row.try_get("updated_at").map_err(errors::any)?,
            })
        }
        Ok(result)
    }

    async fn policy_exist(
        &self,
        id: &str,
        account_id: Option<String>,
        unscoped: bool,
    ) -> Result<bool> {
        let mut wheres = format!(
            r#"`id` = {}"#,
            id.parse::<u64>().map_err(|err| errors::bad_request(&err))?
        );
        if let Some(v) = account_id {
            let account_id_u64: u64 =
                v.parse().map_err(|err| errors::bad_request(&err))?;
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
        .map_err(errors::any)?;
        let count: i64 = result.try_get("count").map_err(errors::any)?;
        Ok(count != 0)
    }

    async fn attach(
        &self,
        id: &str,
        account_id: Option<String>,
        bindings_id: &str,
        bindings_type: BindingsType,
    ) -> Result<()> {
        let set_account_id = |wheres: &mut String,
                              account_id: &Option<String>|
         -> Result<()> {
            if let Some(v) = account_id {
                let account_id_u64: u64 =
                    v.parse().map_err(|err| errors::bad_request(&err))?;
                if !wheres.is_empty() {
                    wheres.push_str(" AND ");
                }
                wheres.push_str(
                    format!(r#"`account_id` = {}"#, account_id_u64).as_str(),
                );
            };
            Ok(())
        };
        match bindings_type {
            BindingsType::User => {
                let mut user_wheres = format!(
                    r#"`id` = {}"#,
                    bindings_id
                        .parse::<u64>()
                        .map_err(|err| errors::bad_request(&err))?
                );
                set_account_id(&mut user_wheres, &account_id)?;

                if sqlx::query(
                    format!(
                        r#"SELECT COUNT(*) as count FROM `user`
            WHERE {} AND `deleted` = 0 LIMIT 1;"#,
                        user_wheres,
                    )
                    .as_str(),
                )
                .fetch_one(&self.pool)
                .await
                .map_err(errors::any)?
                .try_get::<i64, _>("count")
                .map_err(errors::any)?
                    == 0
                {
                    return Err(errors::not_found(&format!(
                        "not found user {}",
                        bindings_id,
                    )));
                }
            }
            BindingsType::Group => {
                let mut group_wheres = format!(
                    r#"`id` = {}"#,
                    bindings_id
                        .parse::<u64>()
                        .map_err(|err| errors::bad_request(&err))?
                );
                set_account_id(&mut group_wheres, &account_id)?;

                if sqlx::query(
                    format!(
                        r#"SELECT COUNT(*) as count FROM `group`
            WHERE {} AND `deleted` = 0 LIMIT 1;"#,
                        group_wheres,
                    )
                    .as_str(),
                )
                .fetch_one(&self.pool)
                .await
                .map_err(errors::any)?
                .try_get::<i64, _>("count")
                .map_err(errors::any)?
                    == 0
                {
                    return Err(errors::not_found(&format!(
                        "not found group {}",
                        id
                    )));
                }
            }
            BindingsType::Role => {
                let mut role_wheres = format!(
                    r#"`id` = {}"#,
                    bindings_id
                        .parse::<u64>()
                        .map_err(|err| errors::bad_request(&err))?
                );
                set_account_id(&mut role_wheres, &account_id)?;

                if sqlx::query(
                    format!(
                        r#"SELECT COUNT(*) as count FROM `role`
            WHERE {} AND `deleted` = 0 LIMIT 1;"#,
                        role_wheres,
                    )
                    .as_str(),
                )
                .fetch_one(&self.pool)
                .await
                .map_err(errors::any)?
                .try_get::<i64, _>("count")
                .map_err(errors::any)?
                    == 0
                {
                    return Err(errors::not_found(&format!(
                        "not found role {}",
                        id
                    )));
                }
            }
        }
        let mut policy_wheres = format!(
            r#"`id` = {}"#,
            id.parse::<u64>().map_err(|err| errors::bad_request(&err))?
        );
        set_account_id(&mut policy_wheres, &account_id)?;
        if sqlx::query(
            format!(
                r#"SELECT COUNT(*) as count FROM `policy`
            WHERE {} AND `deleted` = 0 LIMIT 1;"#,
                policy_wheres,
            )
            .as_str(),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(errors::any)?
        .try_get::<i64, _>("count")
        .map_err(errors::any)?
            == 0
        {
            return Err(errors::not_found(&format!("not found policy {}", id)));
        }
        sqlx::query(
            r#"REPLACE INTO `policy_bindings`
            (`policy_id`,`bindings_type`,`bindings_id`)
            VALUES(?,?,?);"#,
        )
        .bind(id)
        .bind(bindings_type as u8)
        .bind(bindings_id)
        .execute(&self.pool)
        .await
        .map_err(errors::any)?;
        Ok(())
    }
    async fn detach(
        &self,
        id: &str,
        bindings_id: &str,
        bindings_type: BindingsType,
    ) -> Result<()> {
        sqlx::query(
            r#"UPDATE `policy_bindings` SET `deleted` = `id`,`deleted_at`= now()
            WHERE `policy_id` = ? AND bindings_type = ? AND `bindings_id` = ?` AND `deleted` = 0;"#,
        )
        .bind(id.parse::<u64>().map_err(|err| errors::bad_request(&err))?)
        .bind(bindings_type as u8)
        .bind(
            bindings_id
                .parse::<u64>()
                .map_err(|err| errors::bad_request(&err))?,
        )
        .execute(&self.pool)
        .await
        .map_err(errors::any)?;
        Ok(())
    }
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
