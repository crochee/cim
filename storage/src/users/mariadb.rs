use async_trait::async_trait;
use rand::Rng;
use sqlx::{MySqlPool, Row};
use tracing::debug;

use slo::{crypto::password::encrypt, errors, next_id, Result};

use super::{Content, ListOpts, UpdateOpts, User, UserStore};
use crate::{convert::update_set_param, ClaimOpts, List, ID};

#[derive(Clone)]
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

        let mut address = None;
        if let Some(v) = &content.claim.address {
            address = Some(serde_json::to_string(&v).map_err(errors::any)?)
        }

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
            (`id`,`account_id`,`desc`,`email`,`email_verified`,
            `name`,`given_name`,`family_name`,`middle_name`,`nickname`,
            `preferred_username`,`profile`,`picture`,`website`,`gender`,
            `birthday`,`birthdate`,`zoneinfo`,`locale`,`phone_number`,
            `phone_number_verified`,`address`,`secret`,`password`)
            VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?);"#,
        )
        .bind(uid)
        .bind(account_id)
        .bind(&content.desc)
        .bind(&content.claim.email)
        .bind(content.claim.email_verified)
        .bind(&content.claim.name)
        .bind(&content.claim.given_name)
        .bind(&content.claim.family_name)
        .bind(&content.claim.middle_name)
        .bind(&content.claim.nickname)
        .bind(&content.claim.preferred_username)
        .bind(&content.claim.profile)
        .bind(&content.claim.picture)
        .bind(&content.claim.website)
        .bind(&content.claim.gender)
        .bind(&content.claim.birthday)
        .bind(&content.claim.birthdate)
        .bind(&content.claim.zoneinfo)
        .bind(&content.claim.locale)
        .bind(&content.claim.phone_number)
        .bind(content.claim.phone_number_verified)
        .bind(address)
        .bind(secret)
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

        update_set_param(&mut update_content, r#"`desc` = "#, &opts.desc);

        update_set_param(
            &mut update_content,
            r#"`email` = "#,
            &opts.claim.email,
        );

        if let Some(email_verified) = &opts.claim.email_verified {
            update_content.push_str(
                format!(r#"`email_verified` = {} "#, email_verified).as_str(),
            );
        };

        update_set_param(&mut update_content, r#"`name` = "#, &opts.claim.name);
        update_set_param(
            &mut update_content,
            r#"`given_name` = "#,
            &opts.claim.given_name,
        );

        update_set_param(
            &mut update_content,
            r#"`family_name` = "#,
            &opts.claim.family_name,
        );

        update_set_param(
            &mut update_content,
            r#"`middle_name` = "#,
            &opts.claim.middle_name,
        );

        update_set_param(
            &mut update_content,
            r#"`nickname` = "#,
            &opts.claim.nickname,
        );

        update_set_param(
            &mut update_content,
            r#"`preferred_username` = "#,
            &opts.claim.preferred_username,
        );

        update_set_param(
            &mut update_content,
            r#"`profile` = "#,
            &opts.claim.profile,
        );

        update_set_param(
            &mut update_content,
            r#"`picture` = "#,
            &opts.claim.picture,
        );
        update_set_param(
            &mut update_content,
            r#"`website` = "#,
            &opts.claim.website,
        );
        update_set_param(
            &mut update_content,
            r#"`gender` = "#,
            &opts.claim.gender,
        );

        update_set_param(
            &mut update_content,
            r#"`birthday` = "#,
            &opts.claim.birthday,
        );

        update_set_param(
            &mut update_content,
            r#"`birthdate` = "#,
            &opts.claim.birthdate,
        );
        update_set_param(
            &mut update_content,
            r#"`zoneinfo` = "#,
            &opts.claim.zoneinfo,
        );
        update_set_param(
            &mut update_content,
            r#"`locale` = "#,
            &opts.claim.locale,
        );
        update_set_param(
            &mut update_content,
            r#"`phone_number` = "#,
            &opts.claim.phone_number,
        );

        if let Some(phone_number_verified) = &opts.claim.phone_number_verified {
            update_content.push_str(
                format!(
                    r#"`phone_number_verified` = {} "#,
                    phone_number_verified
                )
                .as_str(),
            );
        };
        if let Some(v) = &opts.claim.address {
            let address = serde_json::to_string(v).map_err(errors::any)?;
            update_set_param(
                &mut update_content,
                r#"`address` = "#,
                &Some(address),
            );
        }

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

            update_set_param(
                &mut update_content,
                r#"`secret` = "#,
                &Some(secret),
            );

            update_set_param(
                &mut update_content,
                r#"`password` = "#,
                &Some(password),
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
            format!(
                r#"SELECT `id`,`account_id`,`desc`,`email`,`email_verified`,
                `name`,`given_name`,`family_name`,`middle_name`,`nickname`,
                `preferred_username`,`profile`,`picture`,`website`,`gender`,
                `birthday`,`birthdate`,`zoneinfo`,`locale`,`phone_number`,
                `phone_number_verified`,`address`,`created_at`,`updated_at`
                FROM `user`
                WHERE {} AND `deleted` = 0;"#,
                wheres
            )
            .as_str(),
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
        let mut address = None;
        if let Some(v) = row
            .try_get::<Option<String>, _>("address")
            .map_err(errors::any)?
        {
            address = Some(serde_json::from_str(&v).map_err(errors::any)?);
        }

        Ok(User {
            id: row
                .try_get::<u64, _>("id")
                .map_err(errors::any)?
                .to_string(),
            account_id: row
                .try_get::<u64, _>("account_id")
                .map_err(errors::any)?
                .to_string(),
            created_at: row.try_get("created_at").map_err(errors::any)?,
            updated_at: row.try_get("updated_at").map_err(errors::any)?,
            desc: row.try_get("desc").map_err(errors::any)?,
            claim: ClaimOpts {
                email: row.try_get("email").map_err(errors::any)?,
                email_verified: row
                    .try_get("email_verified")
                    .map_err(errors::any)?,
                name: row.try_get("name").map_err(errors::any)?,
                given_name: row.try_get("given_name").map_err(errors::any)?,
                family_name: row.try_get("family_name").map_err(errors::any)?,
                middle_name: row.try_get("middle_name").map_err(errors::any)?,
                nickname: row.try_get("nickname").map_err(errors::any)?,
                preferred_username: row
                    .try_get("preferred_username")
                    .map_err(errors::any)?,
                profile: row.try_get("profile").map_err(errors::any)?,
                picture: row.try_get("picture").map_err(errors::any)?,
                website: row.try_get("website").map_err(errors::any)?,
                gender: row.try_get("gender").map_err(errors::any)?,
                birthday: row.try_get("birthday").map_err(errors::any)?,
                birthdate: row.try_get("birthdate").map_err(errors::any)?,
                zoneinfo: row.try_get("zoneinfo").map_err(errors::any)?,
                locale: row.try_get("locale").map_err(errors::any)?,
                phone_number: row
                    .try_get("phone_number")
                    .map_err(errors::any)?,
                phone_number_verified: row
                    .try_get("phone_number_verified")
                    .map_err(errors::any)?,
                address,
            },
            secret: None,
            password: None,
        })
    }

    async fn get_user_password(&self, id: &str) -> Result<User> {
        let row = match sqlx::query(
                        r#"SELECT `id`,`account_id`,`desc`,`email`,`email_verified`,
                        `name`,`given_name`,`family_name`,`middle_name`,`nickname`,
                        `preferred_username`,`profile`,`picture`,`website`,`gender`,
                        `birthday`,`birthdate`,`zoneinfo`,`locale`,`phone_number`,
                        `phone_number_verified`,`address`,`password`,`created_at`,`updated_at`
                        FROM `user`
                        WHERE id = ? AND `deleted` = 0;"#,
                        )
                        .bind(id)
                        .fetch_optional(&self.pool).await
        {
            Ok(v) => match v {
                Some(value) => Ok(value),
                None => Err(errors::not_found("no rows")),
            },
            Err(err) => Err(errors::any(err)),
        }?;

        let mut address = None;
        if let Some(v) = row
            .try_get::<Option<String>, _>("address")
            .map_err(errors::any)?
        {
            address = Some(serde_json::from_str(&v).map_err(errors::any)?);
        }

        Ok(User {
            id: row
                .try_get::<u64, _>("id")
                .map_err(errors::any)?
                .to_string(),
            account_id: row
                .try_get::<u64, _>("account_id")
                .map_err(errors::any)?
                .to_string(),
            created_at: row.try_get("created_at").map_err(errors::any)?,
            updated_at: row.try_get("updated_at").map_err(errors::any)?,
            desc: row.try_get("desc").map_err(errors::any)?,
            claim: ClaimOpts {
                email: row.try_get("email").map_err(errors::any)?,
                email_verified: row
                    .try_get("email_verified")
                    .map_err(errors::any)?,
                name: row.try_get("name").map_err(errors::any)?,
                given_name: row.try_get("given_name").map_err(errors::any)?,
                family_name: row.try_get("family_name").map_err(errors::any)?,
                middle_name: row.try_get("middle_name").map_err(errors::any)?,
                nickname: row.try_get("nickname").map_err(errors::any)?,
                preferred_username: row
                    .try_get("preferred_username")
                    .map_err(errors::any)?,
                profile: row.try_get("profile").map_err(errors::any)?,
                picture: row.try_get("picture").map_err(errors::any)?,
                website: row.try_get("website").map_err(errors::any)?,
                gender: row.try_get("gender").map_err(errors::any)?,
                birthday: row.try_get("birthday").map_err(errors::any)?,
                birthdate: row.try_get("birthdate").map_err(errors::any)?,
                zoneinfo: row.try_get("zoneinfo").map_err(errors::any)?,
                locale: row.try_get("locale").map_err(errors::any)?,
                phone_number: row
                    .try_get("phone_number")
                    .map_err(errors::any)?,
                phone_number_verified: row
                    .try_get("phone_number_verified")
                    .map_err(errors::any)?,
                address,
            },
            secret: row.try_get("secret").map_err(errors::any)?,
            password: row.try_get("password").map_err(errors::any)?,
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
        if let Some(group_id) = &filter.group_id {
            if !wheres.is_empty() {
                wheres.push_str(" AND ");
            }
            let group_id_u64: u64 =
                group_id.parse().map_err(|err| errors::bad_request(&err))?;
            wheres.push_str(format!(r#"`id` IN (SELECT `user_id` FROM `group_user` WHERE `group_id` = {} AND `deleted` = 0)"#,
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
                r#"SELECT `id`,`account_id`,`desc`,`email`,`email_verified`,
                `name`,`given_name`,`family_name`,`middle_name`,`nickname`,
                `preferred_username`,`profile`,`picture`,`website`,`gender`,
                `birthday`,`birthdate`,`zoneinfo`,`locale`,`phone_number`,
                `phone_number_verified`,`address`
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
            let mut address = None;
            if let Some(v) = row
                .try_get::<Option<String>, _>("address")
                .map_err(errors::any)?
            {
                address = Some(serde_json::from_str(&v).map_err(errors::any)?);
            }

            result.data.push(User {
                id: row
                    .try_get::<u64, _>("id")
                    .map_err(errors::any)?
                    .to_string(),
                account_id: row
                    .try_get::<u64, _>("account_id")
                    .map_err(errors::any)?
                    .to_string(),
                desc: row.try_get("desc").map_err(errors::any)?,
                claim: ClaimOpts {
                    email: row.try_get("email").map_err(errors::any)?,
                    email_verified: row
                        .try_get("email_verified")
                        .map_err(errors::any)?,
                    name: row.try_get("name").map_err(errors::any)?,
                    given_name: row
                        .try_get("given_name")
                        .map_err(errors::any)?,
                    family_name: row
                        .try_get("family_name")
                        .map_err(errors::any)?,
                    middle_name: row
                        .try_get("middle_name")
                        .map_err(errors::any)?,
                    nickname: row.try_get("nickname").map_err(errors::any)?,
                    preferred_username: row
                        .try_get("preferred_username")
                        .map_err(errors::any)?,
                    profile: row.try_get("profile").map_err(errors::any)?,
                    picture: row.try_get("picture").map_err(errors::any)?,
                    website: row.try_get("website").map_err(errors::any)?,
                    gender: row.try_get("gender").map_err(errors::any)?,
                    birthday: row.try_get("birthday").map_err(errors::any)?,
                    birthdate: row.try_get("birthdate").map_err(errors::any)?,
                    zoneinfo: row.try_get("zoneinfo").map_err(errors::any)?,
                    locale: row.try_get("locale").map_err(errors::any)?,
                    phone_number: row
                        .try_get("phone_number")
                        .map_err(errors::any)?,
                    phone_number_verified: row
                        .try_get("phone_number_verified")
                        .map_err(errors::any)?,
                    address,
                },
                secret: None,
                password: None,
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
