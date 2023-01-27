use async_trait::async_trait;
use chrono::Utc;
use rand::Rng;
use sqlx::{MySqlPool, Row};

use cim_core::{next_id, Error, Result};

use crate::{
    models::{user::User, List, ID},
    pkg::security::encrypt,
};

#[derive(Clone)]
pub struct MariadbUsers {
    pool: MySqlPool,
}

impl MariadbUsers {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl super::UsersRepository for MariadbUsers {
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
        let account_id = match &content.account_id {
            Some(v) => v
                .parse()
                .map_err(|err| Error::BadRequest(format!("{}", err)))?,
            None => uid,
        };
        let nick_name = match &content.nick_name {
            Some(v) => v.to_owned(),
            None => {
                format!("用户{}", uid)
            }
        };
        let secret = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(64)
            .map(char::from)
            .collect::<String>();
        let password = encrypt(&content.password, &secret)?;
        sqlx::query!(
            r#"INSERT INTO `user`
            (`id`,`account_id`,`name`,`nick_name`,`desc`,`email`,`mobile`,`sex`,`image`,`secret`,`password`)
            VALUES(?,?,?,?,?,?,?,?,?,?,?);"#,
            uid,
            account_id,
            content.name,
            nick_name,
            content.desc,
            content.email,
            content.mobile,
            content.sex,
            content.image,
            secret,
            password,
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
        if let Some(nick_name) = &opts.nick_name {
            if !update_content.is_empty() {
                update_content.push_str(" , ");
            }
            update_content
                .push_str(format!(r#"`nick_name` = '{}'"#, nick_name).as_str());
        };
        if let Some(desc) = &opts.desc {
            if !update_content.is_empty() {
                update_content.push_str(" , ");
            }
            update_content.push_str(format!(r#"`desc` = '{}'"#, desc).as_str());
        };
        if let Some(email) = &opts.email {
            if !update_content.is_empty() {
                update_content.push_str(" , ");
            }
            update_content
                .push_str(format!(r#"`email` = '{}'"#, email).as_str());
        };
        if let Some(mobile) = &opts.mobile {
            if !update_content.is_empty() {
                update_content.push_str(" , ");
            }
            update_content
                .push_str(format!(r#"`mobile` = '{}'"#, mobile).as_str());
        };
        if let Some(sex) = &opts.sex {
            if !update_content.is_empty() {
                update_content.push_str(" , ");
            }
            update_content.push_str(format!(r#"`sex` = '{}'"#, sex).as_str());
        };
        if let Some(image) = &opts.image {
            if !update_content.is_empty() {
                update_content.push_str(" , ");
            }
            update_content
                .push_str(format!(r#"`image` = '{}'"#, image).as_str());
        };
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
                r#"UPDATE `user` SET {}
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

    async fn get(&self, id: &str, account_id: Option<String>) -> Result<User> {
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
                None => Err(Error::NotFound("no rows".to_owned())),
            },
            Err(err) => Err(Error::any(err)),
        }?;
        Ok(User {
            id: row.try_get::<u64, _>("id").map_err(Error::any)?.to_string(),
            account_id: row
                .try_get::<u64, _>("account_id")
                .map_err(Error::any)?
                .to_string(),
            name: row.try_get("name").map_err(Error::any)?,
            nick_name: row.try_get("nick_name").map_err(Error::any)?,
            desc: row.try_get("desc").map_err(Error::any)?,
            email: row.try_get("email").map_err(Error::any)?,
            mobile: row.try_get("mobile").map_err(Error::any)?,
            sex: row.try_get("sex").map_err(Error::any)?,
            image: row.try_get("image").map_err(Error::any)?,
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
                r#"UPDATE `user` SET `deleted` = `id`,`deleted_at`= '{}'
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

    async fn list(&self, filter: &super::Querys) -> Result<List<User>> {
        let mut wheres = String::from("");
        if let Some(sex) = &filter.sex {
            wheres.push_str(format!(r#"`sex` = {}"#, sex).as_str());
        };
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
                r#"SELECT COUNT(*) as count FROM `user`
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
                r#"SELECT `id`,`account_id`,`name`,`nick_name`,`desc`,`email`,`mobile`,`sex`,`image`,`created_at`,`updated_at`
                FROM `user`
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
            result.data.push(User {
                id: row
                    .try_get::<u64, _>("id")
                    .map_err(Error::any)?
                    .to_string(),
                account_id: row
                    .try_get::<u64, _>("account_id")
                    .map_err(Error::any)?
                    .to_string(),
                name: row.try_get("name").map_err(Error::any)?,
                nick_name: row.try_get("nick_name").map_err(Error::any)?,
                desc: row.try_get("desc").map_err(Error::any)?,
                email: row.try_get("email").map_err(Error::any)?,
                mobile: row.try_get("mobile").map_err(Error::any)?,
                sex: row.try_get("sex").map_err(Error::any)?,
                image: row.try_get("image").map_err(Error::any)?,
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
                r#"SELECT COUNT(*) as count FROM `user`
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
