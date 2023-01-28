use std::collections::HashSet;

use async_trait::async_trait;
use chrono::Utc;
use sqlx::{MySqlPool, Row};

use cim_core::{next_id, Error, Result};

use crate::models::List;

use super::{
    Content, Kind, Opts, Querys, UserGroupBinding, UserGroupBindingsRepository,
};

#[derive(Clone)]
pub struct MariadbUserGroupBindings {
    pool: MySqlPool,
}

impl MariadbUserGroupBindings {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserGroupBindingsRepository for MariadbUserGroupBindings {
    async fn create(
        &self,
        account_id: String,
        content: &Content,
    ) -> Result<()> {
        let mut temp = HashSet::new();
        for item in content.items.iter() {
            temp.insert((
                item.user_group_id.clone(),
                item.subject_id.clone(),
                item.kind.clone(),
            ));
        }
        let mut tx = self.pool.begin().await.map_err(Error::any)?;
        let mut temp_exist = HashSet::new();
        for value in temp {
            if !temp_exist.contains(&value.0) {
                temp_exist.insert(value.0.clone());
                if sqlx::query!(
                    r#"SELECT `id` as count FROM `user_group`
                    WHERE `id` = ? AND `account_id` = ? AND `deleted` = 0 LIMIT 1;"#,
                    value.0,
                    account_id,
                )
                .fetch_optional(&mut tx)
                .await
                .map_err(Error::any)?
                .is_none()
                {
                    return Err(Error::NotFound(format!(
                        "not found user_group {}",
                        value.0,
                    )));
                }
            }

            match value.2 {
                Kind::User => {
                    if !temp_exist.contains(&value.1) {
                        temp_exist.insert(value.1.clone());
                        if sqlx::query!(
                            r#"SELECT `id` as count FROM `user`
                            WHERE `id` = ? AND `account_id` = ? AND `deleted` = 0 LIMIT 1;"#,
                            value.1,
                            account_id,
                        )
                        .fetch_optional(&mut tx)
                        .await
                        .map_err(Error::any)?
                        .is_none()
                        {
                            return Err(Error::NotFound(format!(
                                "not found user {}",
                                value.1
                            )));
                        }
                    }

                    sqlx::query!(
                        r#"INSERT INTO `user_group_user`
                        (`id`,`user_group_id`,`user_id`)
                        VALUES(?,?,?);"#,
                        next_id().map_err(Error::any)?,
                        value.0,
                        value.1,
                    )
                    .execute(&mut tx)
                    .await
                    .map_err(Error::any)?;
                }
                Kind::Role => {
                    if !temp_exist.contains(&value.1) {
                        temp_exist.insert(value.1.clone());
                        if sqlx::query!(
                            r#"SELECT `id` as count FROM `role`
                            WHERE `id` = ? AND `account_id` = ? AND `deleted` = 0 LIMIT 1;"#,
                            value.1,
                            account_id,
                        )
                        .fetch_optional(&mut tx)
                        .await
                        .map_err(Error::any)?
                        .is_none()
                        {
                            return Err(Error::NotFound(format!(
                                "not found role {}",
                                value.1
                            )));
                        }
                    }
                    sqlx::query!(
                        r#"INSERT INTO `user_group_role`
                        (`id`,`user_group_id`,`role_id`)
                        VALUES(?,?,?);"#,
                        next_id().map_err(Error::any)?,
                        value.0,
                        value.1,
                    )
                    .execute(&mut tx)
                    .await
                    .map_err(Error::any)?;
                }
            }
        }
        tx.commit().await.map_err(Error::any)?;
        Ok(())
    }

    async fn delete(&self, opts: &Opts) -> Result<()> {
        let mut temp = HashSet::new();
        for item in opts.items.iter() {
            temp.insert((item.id.clone(), item.kind.clone()));
        }
        let mut tx = self.pool.begin().await.map_err(Error::any)?;
        for value in temp {
            match value.1 {
                Kind::User => {
                    sqlx::query!(
                        r#"UPDATE `user_group_user` SET `deleted` = `id`,`deleted_at`= ?
                        WHERE `id` = ? AND `deleted` = 0;"#,
                        Utc::now().naive_utc(),
                        value.0,
                    )
                    .execute(&mut tx)
                    .await
                    .map_err(Error::any)?;
                }
                Kind::Role => {
                    sqlx::query!(
                        r#"UPDATE `user_group_role` SET `deleted` = `id`,`deleted_at`= ?
                        WHERE `id` = ? AND `deleted` = 0;"#,
                        Utc::now().naive_utc(),
                        value.0,
                    )
                    .execute(&mut tx)
                    .await
                    .map_err(Error::any)?;
                }
            }
        }
        tx.commit().await.map_err(Error::any)?;
        Ok(())
    }

    async fn list(&self, filter: &Querys) -> Result<List<UserGroupBinding>> {
        let mut wheres = String::from("");
        if let Some(user_group_id) = &filter.user_group_id {
            wheres.push_str(
                format!(r#"`user_group_id` = {}"#, user_group_id).as_str(),
            );
        };
        if !wheres.is_empty() {
            wheres.push_str(" AND ");
        }
        wheres.push_str(r#"`deleted` = 0"#);

        let mut result = List {
            data: Vec::new(),
            limit: filter.pagination.limit,
            offset: filter.pagination.offset,
            total: 0,
        };

        match &filter.kind {
            None => {
                self.list_user_group_user(&wheres, &mut result).await?;
                self.list_user_group_role(&wheres, &mut result).await?;
            }
            Some(value) => match value {
                Kind::User => {
                    self.list_user_group_user(&wheres, &mut result).await?;
                }
                Kind::Role => {
                    self.list_user_group_role(&wheres, &mut result).await?;
                }
            },
        }
        Ok(result)
    }
}

impl MariadbUserGroupBindings {
    async fn list_user_group_user(
        &self,
        wheres: &str,
        result: &mut List<UserGroupBinding>,
    ) -> Result<()> {
        let user_role_result = sqlx::query(
            format!(
                r#"SELECT COUNT(*) as count FROM `user_group_user`
            WHERE {};"#,
                wheres,
            )
            .as_str(),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::any)?;
        result.total += user_role_result
            .try_get::<i64, _>("count")
            .map_err(Error::any)?;

        let rows = sqlx::query(
            format!(
                r#"SELECT `id`,`user_group_id`,`user_id`
                FROM `user_group_user`
                WHERE {};"#,
                wheres,
            )
            .as_str(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::any)?;
        for row in rows.iter() {
            result.data.push(UserGroupBinding {
                id: row
                    .try_get::<u64, _>("id")
                    .map_err(Error::any)?
                    .to_string(),
                user_group_id: row
                    .try_get("user_group_id")
                    .map_err(Error::any)?,
                kind: Kind::User,
                subject_id: row.try_get("user_id").map_err(Error::any)?,
            })
        }
        Ok(())
    }
    async fn list_user_group_role(
        &self,
        wheres: &str,
        result: &mut List<UserGroupBinding>,
    ) -> Result<()> {
        let user_role_result = sqlx::query(
            format!(
                r#"SELECT COUNT(*) as count FROM `user_group_role`
            WHERE {};"#,
                wheres,
            )
            .as_str(),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::any)?;
        result.total += user_role_result
            .try_get::<i64, _>("count")
            .map_err(Error::any)?;

        let rows = sqlx::query(
            format!(
                r#"SELECT `id`,`user_group_id`,`role_id`
                FROM `user_group_role`
                WHERE {};"#,
                wheres,
            )
            .as_str(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::any)?;
        for row in rows.iter() {
            result.data.push(UserGroupBinding {
                id: row
                    .try_get::<u64, _>("id")
                    .map_err(Error::any)?
                    .to_string(),
                user_group_id: row
                    .try_get("user_group_id")
                    .map_err(Error::any)?,
                kind: Kind::Role,
                subject_id: row.try_get("role_id").map_err(Error::any)?,
            })
        }
        Ok(())
    }
}
