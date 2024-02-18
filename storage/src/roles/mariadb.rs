use async_trait::async_trait;
use sqlx::{MySqlPool, Row};

use slo::{errors, next_id, Result};

use crate::{
    convert::{convert_field, update_set_param},
    List, ID,
};

use super::{Content, ListOpts, Role, RoleStore, UpdateOpts, UserType};

pub struct RoleImpl {
    pool: MySqlPool,
}

impl RoleImpl {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RoleStore for RoleImpl {
    // roles
    async fn create_role(
        &self,
        id: Option<String>,
        content: &Content,
    ) -> Result<ID> {
        let uid = match id {
            Some(v) => v.parse().map_err(|err| errors::bad_request(&err))?,
            None => next_id().map_err(errors::any)?,
        };
        let account_id: u64 = content
            .account_id
            .parse()
            .map_err(|err| errors::bad_request(&err))?;
        sqlx::query(
            r#"INSERT INTO `role`
            (`id`,`account_id`,`name`,`desc`)
            VALUES(?,?,?,?);"#,
        )
        .bind(uid)
        .bind(account_id)
        .bind(convert_field(&content.name))
        .bind(convert_field(&content.desc))
        .execute(&self.pool)
        .await
        .map_err(errors::any)?;

        Ok(ID {
            id: uid.to_string(),
        })
    }
    async fn update_role(
        &self,
        id: &str,
        account_id: Option<String>,
        opts: &UpdateOpts,
    ) -> Result<()> {
        let mut update_content = String::new();

        update_set_param(&mut update_content, r#"`name` = "#, &opts.name);

        update_set_param(&mut update_content, r#"`desc` = "#, &opts.desc);

        if update_content.is_empty() {
            return Ok(());
        }
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
                r#"UPDATE `role` SET {}
                WHERE {};"#,
                update_content, wheres,
            )
            .as_str(),
        )
        .execute(&self.pool)
        .await
        .map_err(errors::any)?;
        Ok(())
    }
    async fn get_role(
        &self,
        id: &str,
        account_id: Option<String>,
    ) -> Result<Role> {
        let mut wheres = format!(
            r#"`id` = {}"#,
            id.parse::<u64>().map_err(|err| errors::bad_request(&err))?
        );
        if let Some(v) = &account_id {
            let account_id_u64: u64 =
                v.parse().map_err(|err| errors::bad_request(&err))?;
            wheres.push_str(" AND ");
            wheres.push_str(
                format!(r#"`account_id` = {}"#, account_id_u64).as_str(),
            );
        }

        let row = match sqlx::query(
            format!(r#"SELECT `id`,`account_id`,`name`,`desc`,`created_at`,`updated_at`
            FROM `role`
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
        Ok(Role {
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
        })
    }
    async fn delete_role(
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
            r#"SELECT COUNT(*) as count FROM `role_bindings` WHERE `role_id` = ? AND `deleted` = 0"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(errors::any)?.try_get::<i64,_>("count").map_err(errors::any)?!=0{
            return Err(errors::forbidden(&"can't delete role, because it is used by user".to_string()));
        };
        if sqlx::query(
            r#"SELECT COUNT(*) as count FROM `policy_bindings` WHERE `bindings_type` = 3 AND `bindings_id` = ? AND `deleted` = 0"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(errors::any)?.try_get::<i64,_>("count").map_err(errors::any)?!=0{
            return Err(errors::forbidden(&"can't delete role, because it is attached by policy".to_string()));
        };
        sqlx::query(
            format!(
                r#"UPDATE `role` SET `deleted` = `id`,`deleted_at`= now()
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
    async fn list_role(&self, filter: &ListOpts) -> Result<List<Role>> {
        let mut wheres = String::new();
        if let Some(account_id) = &filter.account_id {
            let account_id_u64: u64 = account_id
                .parse()
                .map_err(|err| errors::bad_request(&err))?;
            if !wheres.is_empty() {
                wheres.push_str(" AND ");
            }
            wheres.push_str(
                format!(r#"`account_id` = {}"#, account_id_u64).as_str(),
            );
        };
        if let Some(user_id) = &filter.user_id {
            if !wheres.is_empty() {
                wheres.push_str(" AND ");
            }
            wheres.push_str(format!(r#"`id` IN (SELECT `role_id` FROM `role_bindings` WHERE `user_id` = {})"#,
            convert_field(user_id)).as_str());
        }
        if !wheres.is_empty() {
            wheres.push_str(" AND ");
        }
        wheres.push_str(r#"`deleted` = 0"#);
        // 查询total
        let policy_result = sqlx::query(
            format!(
                r#"SELECT COUNT(*) as count FROM `role`
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
                r#"SELECT `id`,`account_id`,`name`,`desc`,`created_at`,`updated_at`
                FROM `role`
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
            result.data.push(super::Role {
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
            })
        }
        Ok(result)
    }
    async fn role_exist(
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
                r#"SELECT COUNT(*) as count FROM `role`
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
    async fn attach_user(
        &self,
        id: &str,
        account_id: Option<String>,
        user_id: &str,
        user_type: UserType,
    ) -> Result<()> {
        let mut role_wheres = format!(
            r#"`id` = {}"#,
            id.parse::<u64>().map_err(|err| errors::bad_request(&err))?
        );
        if let Some(v) = &account_id {
            let account_id_u64: u64 =
                v.parse().map_err(|err| errors::bad_request(&err))?;
            role_wheres.push_str(" AND ");
            role_wheres.push_str(
                format!(r#"`account_id` = {}"#, account_id_u64).as_str(),
            );
        }
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
            return Err(errors::not_found(&format!("not found role {}", id,)));
        }
        sqlx::query(
            r#"REPLACE INTO `role_bindings`
            (`role_id`,`user_type`,`user_id`)
            VALUES(?,?,?);"#,
        )
        .bind(id)
        .bind(user_type as u8)
        .bind(convert_field(user_id))
        .execute(&self.pool)
        .await
        .map_err(errors::any)?;
        Ok(())
    }
    async fn detach_user(
        &self,
        id: &str,
        user_id: &str,
        user_type: UserType,
    ) -> Result<()> {
        sqlx::query(
            r#"UPDATE `role_bindings` SET `deleted` = `id`,`deleted_at`= now()
            WHERE `role_id` = ? AND `user_type` = ? AND `user_id` = ? AND `deleted` = 0;"#,
        )
        .bind(id.parse::<u64>().map_err(|err| errors::bad_request(&err))?)
        .bind(user_type as u8)
        .bind(convert_field(user_id))
        .execute(&self.pool)
        .await
        .map_err(errors::any)?;
        Ok(())
    }
}
