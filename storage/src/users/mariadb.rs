use async_trait::async_trait;
use rand::Rng;
use sqlx::{MySqlPool, Row};
use tracing::debug;

use slo::{crypto::password::encrypt, errors, next_id, Result};

use super::{
    nick_name_generator, Content, ListOpts, UpdateOpts, User, UserStore,
};
use crate::{
    convert::{convert_field, update_set_param},
    List, ID,
};

pub struct UserImpl {
    pool: MySqlPool,
}

impl UserImpl {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserStore for UserImpl {
    async fn create_user(
        &self,
        id: Option<String>,
        content: &Content,
    ) -> Result<ID> {
        debug!("create user {:#?}", content);
        let uid = match id {
            Some(v) => v.parse().map_err(|err| errors::bad_request(&err))?,
            None => next_id().map_err(errors::any)?,
        };
        let account_id = match &content.account_id {
            Some(v) => v.parse().map_err(|err| errors::bad_request(&err))?,
            None => uid,
        };
        let nick_name = match &content.nick_name {
            Some(v) => convert_field(v),
            None => nick_name_generator(&content.name),
        };
        debug!("start create secret");
        let secret = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(64)
            .map(char::from)
            .collect::<String>();
        debug!("end create secret {}", secret);
        let password = encrypt(&content.password, &secret)?;
        debug!("end encrypt password");
        sqlx::query(
            r#"INSERT INTO `user`
            (`id`,`account_id`,`name`,`nick_name`,`desc`,`email`,`mobile`,`sex`,`image`,`password`)
            VALUES(?,?,?,?,?,?,?,?,?,?);"#,
            )
           .bind(uid)
           .bind(account_id)
           .bind(&content.name)
           .bind(nick_name)
           .bind(&content.desc)
           .bind(&content.email)
           .bind(&content.mobile)
           .bind(&content.sex)
           .bind(&content.image)
           .bind(password)
        .execute(&self.pool)
        .await
        .map_err(errors::any)?;

        Ok(ID {
            id: uid.to_string(),
        })
    }

    async fn update_user(
        &self,
        id: &str,
        account_id: Option<String>,
        opts: &UpdateOpts,
    ) -> Result<()> {
        // update set 构造
        let mut update_content = String::new();

        update_set_param(&mut update_content, r#"`name` = "#, &opts.name);

        update_set_param(
            &mut update_content,
            r#"`nick_name` = "#,
            &opts.nick_name,
        );

        update_set_param(&mut update_content, r#"`desc` = "#, &opts.desc);

        update_set_param(&mut update_content, r#"`email` = "#, &opts.email);

        update_set_param(&mut update_content, r#"`mobile` = "#, &opts.mobile);

        update_set_param(&mut update_content, r#"`sex` = "#, &opts.sex);

        update_set_param(&mut update_content, r#"`image` = "#, &opts.image);

        if let Some(password_value) = &opts.password {
            if !update_content.is_empty() {
                update_content.push_str(" , ");
            }
            let secret = rand::thread_rng()
                .sample_iter(&rand::distributions::Alphanumeric)
                .take(64)
                .map(char::from)
                .collect::<String>();
            let password = encrypt(password_value, &secret)?;

            update_content.push_str(
                format!(
                    r#"`secret` = '{}' , `password` = '{}'"#,
                    secret, password
                )
                .as_str(),
            );
        };
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
                r#"UPDATE `user` SET {}
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

    async fn get_user(
        &self,
        id: &str,
        account_id: Option<String>,
    ) -> Result<User> {
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
            format!(r#"SELECT `id`,`account_id`,`name`,`nick_name`,`desc`,`email`,`mobile`,`sex`,`image`,`created_at`,`updated_at`
            FROM `user`
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
        Ok(User {
            id: row
                .try_get::<u64, _>("id")
                .map_err(errors::any)?
                .to_string(),
            account_id: row
                .try_get::<u64, _>("account_id")
                .map_err(errors::any)?
                .to_string(),
            name: row.try_get("name").map_err(errors::any)?,
            nick_name: row.try_get("nick_name").map_err(errors::any)?,
            desc: row.try_get("desc").map_err(errors::any)?,
            email: row.try_get("email").map_err(errors::any)?,
            mobile: row.try_get("mobile").map_err(errors::any)?,
            sex: row.try_get("sex").map_err(errors::any)?,
            image: row.try_get("image").map_err(errors::any)?,
            created_at: row.try_get("created_at").map_err(errors::any)?,
            updated_at: row.try_get("updated_at").map_err(errors::any)?,
        })
    }

    async fn delete_user(
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
            r#"SELECT COUNT(*) as count FROM `group_user` WHERE `user_id` = ? AND `deleted` = 0"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(errors::any)?.try_get::<i64,_>("count").map_err(errors::any)?!=0{
            return Err(errors::forbidden(&"can't delete user, because it is attached to group".to_string()));
        };
        if sqlx::query(
            r#"SELECT COUNT(*) as count FROM `policy_bindings` WHERE `bindings_type` = 1 AND `bindings_id` = ? AND `deleted` = 0"#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(errors::any)?.try_get::<i64,_>("count").map_err(errors::any)?!=0{
            return Err(errors::forbidden(&"can't delete user, because it is attached by policy".to_string()));
        };
        sqlx::query(
            format!(
                r#"UPDATE `user` SET `deleted` = `id`,`deleted_at`= now()
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

    async fn list_user(&self, filter: &ListOpts) -> Result<List<User>> {
        let mut wheres = String::new();
        if let Some(account_id) = &filter.account_id {
            let account_id_u64: u64 = account_id
                .parse()
                .map_err(|err| errors::bad_request(&err))?;

            wheres.push_str(
                format!(r#"`account_id` = {}"#, account_id_u64).as_str(),
            );
        }
        if let Some(sex) = &filter.sex {
            if !wheres.is_empty() {
                wheres.push_str(" AND ");
            }
            wheres.push_str(r#"`sex` = "#);
            wheres.push_str(sex);
        };
        if let Some(group_id) = &filter.group_id {
            if !wheres.is_empty() {
                wheres.push_str(" AND ");
            }
            let group_id_u64: u64 =
                group_id.parse().map_err(|err| errors::bad_request(&err))?;
            wheres.push_str(format!(r#"`id` IN (SELECT `user_id` FROM `group_user` WHERE `group_id` = {})"#,
            group_id_u64).as_str());
        };

        if !wheres.is_empty() {
            wheres.push_str(" AND ");
        }
        wheres.push_str(r#"`deleted` = 0"#);
        // 查询total
        let policy_result = sqlx::query(
            format!(
                r#"SELECT COUNT(*) as count FROM `user`
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
                r#"SELECT `id`,`account_id`,`name`,`nick_name`,`desc`,`email`,`mobile`,`sex`,`image`,`created_at`,`updated_at`
                FROM `user`
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
            result.data.push(User {
                id: row
                    .try_get::<u64, _>("id")
                    .map_err(errors::any)?
                    .to_string(),
                account_id: row
                    .try_get::<u64, _>("account_id")
                    .map_err(errors::any)?
                    .to_string(),
                name: row.try_get("name").map_err(errors::any)?,
                nick_name: row.try_get("nick_name").map_err(errors::any)?,
                desc: row.try_get("desc").map_err(errors::any)?,
                email: row.try_get("email").map_err(errors::any)?,
                mobile: row.try_get("mobile").map_err(errors::any)?,
                sex: row.try_get("sex").map_err(errors::any)?,
                image: row.try_get("image").map_err(errors::any)?,
                created_at: row.try_get("created_at").map_err(errors::any)?,
                updated_at: row.try_get("updated_at").map_err(errors::any)?,
            })
        }
        Ok(result)
    }

    async fn user_exist(
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
                r#"SELECT COUNT(*) as count FROM `user`
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
}
