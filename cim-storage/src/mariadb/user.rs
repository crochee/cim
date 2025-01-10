use async_trait::async_trait;
use rand::Rng;
use sqlx::{MySqlPool, Row};

use cim_slo::{crypto::password::encrypt, errors, Result};
use tracing::info;

use crate::{
    user::{ListParams, User},
    ClaimOpts, Interface, List,
};

#[derive(Clone, Debug)]
pub struct UserImpl {
    pool: MySqlPool,
}

impl UserImpl {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Interface for UserImpl {
    type T = User;
    type L = ListParams;

    #[tracing::instrument]
    async fn put(&self, input: &Self::T) -> Result<()> {
        let mut address = None;
        if let Some(v) = &input.claim.address {
            address = Some(serde_json::to_string(&v).map_err(errors::any)?)
        }

        let secret = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(64)
            .map(char::from)
            .collect::<String>();
        let temp_password = input
            .password
            .as_ref()
            .ok_or_else(|| errors::bad_request("password is required"))?;
        info!("password: {}", temp_password);
        let password = encrypt(temp_password, &secret)?;

        sqlx::query(
            r#"REPLACE INTO `user`
            (`id`,`account_id`,`desc`,`email`,`email_verified`,
            `name`,`given_name`,`family_name`,`middle_name`,`nickname`,
            `preferred_username`,`profile`,`picture`,`website`,`gender`,
            `birthday`,`birthdate`,`zoneinfo`,`locale`,`phone_number`,
            `phone_number_verified`,`address`,`secret`,`password`)
            VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?);"#,
        )
        .bind(&input.id)
        .bind(&input.account_id)
        .bind(&input.desc)
        .bind(&input.claim.email)
        .bind(input.claim.email_verified)
        .bind(&input.claim.name)
        .bind(&input.claim.given_name)
        .bind(&input.claim.family_name)
        .bind(&input.claim.middle_name)
        .bind(&input.claim.nickname)
        .bind(&input.claim.preferred_username)
        .bind(&input.claim.profile)
        .bind(&input.claim.picture)
        .bind(&input.claim.website)
        .bind(&input.claim.gender)
        .bind(&input.claim.birthday)
        .bind(&input.claim.birthdate)
        .bind(&input.claim.zoneinfo)
        .bind(&input.claim.locale)
        .bind(&input.claim.phone_number)
        .bind(input.claim.phone_number_verified)
        .bind(address)
        .bind(secret)
        .bind(password)
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
            r#"UPDATE `user` SET `deleted` = `id`,`deleted_at`= now()
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
                r#"SELECT `id`,`account_id`,`desc`,`email`,`email_verified`,
                `name`,`given_name`,`family_name`,`middle_name`,`nickname`,
                `preferred_username`,`profile`,`picture`,`website`,`gender`,
                `birthday`,`birthdate`,`zoneinfo`,`locale`,`phone_number`,
                `phone_number_verified`,`address`,`secret`,`password`,`created_at`,`updated_at`
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

        output.id = row
            .try_get::<u64, _>("id")
            .map_err(errors::any)?
            .to_string();
        output.account_id = row
            .try_get::<u64, _>("account_id")
            .map_err(errors::any)?
            .to_string();
        output.created_at = row.try_get("created_at").map_err(errors::any)?;
        output.updated_at = row.try_get("updated_at").map_err(errors::any)?;
        output.desc = row.try_get("desc").map_err(errors::any)?;
        output.claim = ClaimOpts {
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
            phone_number: row.try_get("phone_number").map_err(errors::any)?,
            phone_number_verified: row
                .try_get("phone_number_verified")
                .map_err(errors::any)?,
            address,
        };
        output.secret = row.try_get("secret").map_err(errors::any)?;
        output.password = row.try_get("password").map_err(errors::any)?;
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
                    r#"SELECT COUNT(*) as count FROM `user`
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
                r#"SELECT `id`,`account_id`,`desc`,`email`,`email_verified`,
                `name`,`given_name`,`family_name`,`middle_name`,`nickname`,
                `preferred_username`,`profile`,`picture`,`website`,`gender`,
                `birthday`,`birthdate`,`zoneinfo`,`locale`,`phone_number`,
                `phone_number_verified`,`address`,`created_at`,`updated_at`
                FROM `user`
                WHERE {};"#,
                wheres,
            )
            .as_str(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(errors::any)?;
        for row in rows.iter() {
            let mut address = None;
            if let Some(v) = row
                .try_get::<Option<String>, _>("address")
                .map_err(errors::any)?
            {
                address = Some(serde_json::from_str(&v).map_err(errors::any)?);
            }

            output.data.push(Self::T {
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
                r#"SELECT COUNT(*) as count FROM `user`
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

    if let Some(account_id) = &opts.account_id {
        let account_id_u64: u64 = account_id
            .parse()
            .map_err(|err| errors::bad_request(&err))?;

        if !wheres.is_empty() {
            wheres.push_str(" AND ");
        }
        wheres
            .push_str(format!(r#"`account_id` = {}"#, account_id_u64).as_str());
    }

    if let Some(group_id) = &opts.group_id {
        if !wheres.is_empty() {
            wheres.push_str(" AND ");
        }
        let group_id_u64: u64 =
            group_id.parse().map_err(|err| errors::bad_request(&err))?;
        wheres.push_str(format!(r#"`id` IN (SELECT `user_id` FROM `group_user` WHERE `group_id` = {} AND `deleted` = 0)"#,
            group_id_u64).as_str());
    };
    Ok(())
}
