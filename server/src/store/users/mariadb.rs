use chrono::Utc;
use rand::Rng;
use sqlx::{MySqlPool, Row};

use cim_core::{next_id, Code, Result};

use crate::{
    models::{user::User, List, ID},
    pkg::security::encrypt,
};

use super::{Password, UserSubject};

pub async fn create(
    pool: &MySqlPool,
    id: Option<String>,
    content: &super::Content,
) -> Result<ID> {
    let uid = match id {
        Some(v) => v.parse().map_err(|err| Code::bad_request(&err))?,
        None => next_id().map_err(Code::any)?,
    };
    let account_id = match &content.account_id {
        Some(v) => v.parse().map_err(|err| Code::bad_request(&err))?,
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
        .execute(pool)
        .await
        .map_err(Code::any)?;

    Ok(ID {
        id: uid.to_string(),
    })
}

pub async fn update(
    pool: &MySqlPool,
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
        update_content.push_str(format!(r#"`email` = '{}'"#, email).as_str());
    };
    if let Some(mobile) = &opts.mobile {
        if !update_content.is_empty() {
            update_content.push_str(" , ");
        }
        update_content.push_str(format!(r#"`mobile` = '{}'"#, mobile).as_str());
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
        update_content.push_str(format!(r#"`image` = '{}'"#, image).as_str());
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
            format!(r#"`secret` = '{}' , `password` = '{}'"#, secret, password)
                .as_str(),
        );
    };
    if update_content.is_empty() {
        return Ok(());
    }
    let mut wheres = format!(r#"`id` = {}"#, id);
    if let Some(v) = account_id {
        let account_id_u64: u64 =
            v.parse().map_err(|err| Code::bad_request(&err))?;
        wheres.push_str(" AND ");
        wheres
            .push_str(format!(r#"`account_id` = {}"#, account_id_u64).as_str());
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
    .execute(pool)
    .await
    .map_err(Code::any)?;
    Ok(())
}

pub async fn get(
    pool: &MySqlPool,
    id: &str,
    account_id: Option<String>,
) -> Result<User> {
    let mut wheres = format!(r#"`id` = {}"#, id);
    if let Some(v) = account_id {
        let account_id_u64: u64 =
            v.parse().map_err(|err| Code::bad_request(&err))?;
        wheres.push_str(" AND ");
        wheres
            .push_str(format!(r#"`account_id` = {}"#, account_id_u64).as_str());
    }

    let row = match sqlx::query(
            format!(r#"SELECT `id`,`account_id`,`name`,`nick_name`,`desc`,`email`,`mobile`,`sex`,`image`,`created_at`,`updated_at`
            FROM `user`
            WHERE {} AND `deleted` = 0;"#,
            wheres)
            .as_str()
        )
        .fetch_optional(pool)
        .await
        {
            Ok(v) => match v {
                Some(value) => Ok(value),
                None => Err(Code::not_found("no rows")),
            },
            Err(err) => Err(Code::any(err)),
        }?;
    Ok(User {
        id: row.try_get::<u64, _>("id").map_err(Code::any)?.to_string(),
        account_id: row
            .try_get::<u64, _>("account_id")
            .map_err(Code::any)?
            .to_string(),
        name: row.try_get("name").map_err(Code::any)?,
        nick_name: row.try_get("nick_name").map_err(Code::any)?,
        desc: row.try_get("desc").map_err(Code::any)?,
        email: row.try_get("email").map_err(Code::any)?,
        mobile: row.try_get("mobile").map_err(Code::any)?,
        sex: row.try_get("sex").map_err(Code::any)?,
        image: row.try_get("image").map_err(Code::any)?,
        created_at: row.try_get("created_at").map_err(Code::any)?,
        updated_at: row.try_get("updated_at").map_err(Code::any)?,
    })
}

pub async fn delete(
    pool: &MySqlPool,
    id: &str,
    account_id: Option<String>,
) -> Result<()> {
    let mut wheres = format!(r#"`id` = {}"#, id);
    if let Some(v) = account_id {
        let account_id_u64: u64 =
            v.parse().map_err(|err| Code::bad_request(&err))?;
        wheres.push_str(" AND ");
        wheres
            .push_str(format!(r#"`account_id` = {}"#, account_id_u64).as_str());
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
    .execute(pool)
    .await
    .map_err(Code::any)?;
    Ok(())
}

pub async fn list(
    pool: &MySqlPool,
    filter: &super::Querys,
) -> Result<List<User>> {
    let mut wheres = String::from("");
    if let Some(sex) = &filter.sex {
        wheres.push_str(format!(r#"`sex` = {}"#, sex).as_str());
    };
    if let Some(account_id) = &filter.account_id {
        let account_id_u64: u64 =
            account_id.parse().map_err(|err| Code::bad_request(&err))?;
        if !wheres.is_empty() {
            wheres.push_str(" AND ");
        }
        wheres
            .push_str(format!(r#"`account_id` = {}"#, account_id_u64).as_str());
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
    .fetch_one(pool)
    .await
    .map_err(Code::any)?;
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
        .fetch_all(pool)
        .await
        .map_err(Code::any)?;
    let mut result = List {
        data: Vec::new(),
        limit: filter.pagination.limit,
        offset: filter.pagination.offset,
        total: policy_result.try_get("count").map_err(Code::any)?,
    };
    for row in rows.iter() {
        result.data.push(User {
            id: row.try_get::<u64, _>("id").map_err(Code::any)?.to_string(),
            account_id: row
                .try_get::<u64, _>("account_id")
                .map_err(Code::any)?
                .to_string(),
            name: row.try_get("name").map_err(Code::any)?,
            nick_name: row.try_get("nick_name").map_err(Code::any)?,
            desc: row.try_get("desc").map_err(Code::any)?,
            email: row.try_get("email").map_err(Code::any)?,
            mobile: row.try_get("mobile").map_err(Code::any)?,
            sex: row.try_get("sex").map_err(Code::any)?,
            image: row.try_get("image").map_err(Code::any)?,
            created_at: row.try_get("created_at").map_err(Code::any)?,
            updated_at: row.try_get("updated_at").map_err(Code::any)?,
        })
    }
    Ok(result)
}

pub async fn exist(
    pool: &MySqlPool,
    id: &str,
    account_id: Option<String>,
    unscoped: bool,
) -> Result<bool> {
    let mut wheres = format!(r#"`id` = {}"#, id);
    if let Some(v) = account_id {
        let account_id_u64: u64 =
            v.parse().map_err(|err| Code::bad_request(&err))?;
        wheres.push_str(" AND ");
        wheres
            .push_str(format!(r#"`account_id` = {}"#, account_id_u64).as_str());
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
    .fetch_one(pool)
    .await
    .map_err(Code::any)?;
    let count: i64 = result.try_get("count").map_err(Code::any)?;
    Ok(count != 0)
}

pub async fn get_password(
    pool: &MySqlPool,
    value: &UserSubject,
) -> Result<Password> {
    let wheres = match value {
        UserSubject::UserID(id) => format!("`id` = {}", id),
        UserSubject::Email(email) => format!("`email` = '{}'", email),
        UserSubject::Mobile(mobile) => format!("`mobile` = '{}'", mobile),
    };
    let row = match sqlx::query(
            format!(
                r#"SELECT `id`,`name`,`nick_name`,`email`,`mobile`,`secret`,`password`
                FROM `user`
                WHERE {}  AND `deleted` = 0;"#,
                wheres,
            )
            .as_str(),
        )
        .fetch_optional(pool)
        .await
        {
            Ok(v) => match v {
                Some(value) => Ok(value),
                None => Err(Code::not_found("no rows")),
            },
            Err(err) => Err(Code::any(err)),
        }?;

    Ok(Password {
        user_id: row.try_get::<u64, _>("id").map_err(Code::any)?.to_string(),
        user_name: row.try_get("name").map_err(Code::any)?,
        nick_name: row.try_get("nick_name").map_err(Code::any)?,
        email: row.try_get("email").map_err(Code::any)?,
        mobile: row.try_get("mobile").map_err(Code::any)?,
        hash: row.try_get("password").map_err(Code::any)?,
        secret: row.try_get("secret").map_err(Code::any)?,
    })
}
