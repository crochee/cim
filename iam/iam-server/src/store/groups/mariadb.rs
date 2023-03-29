use chrono::Utc;
use sqlx::{MySqlPool, Row};

use cim_core::{next_id, Code, Result};

use crate::models::{
    usergroup::{Kind, UserGroup, UserGroupBinding, UserGroupBindings},
    List, ID,
};

pub async fn create(
    pool: &MySqlPool,
    id: Option<String>,
    content: &super::Content,
) -> Result<ID> {
    let uid = match id {
        Some(v) => v.parse().map_err(|err| Code::bad_request(&err))?,
        None => next_id().map_err(Code::any)?,
    };
    let account_id: u64 = content
        .account_id
        .parse()
        .map_err(|err| Code::bad_request(&err))?;
    let user_id: u64 = content
        .user_id
        .parse()
        .map_err(|err| Code::bad_request(&err))?;
    sqlx::query!(
        r#"INSERT INTO `group`
            (`id`,`account_id`,`user_id`,`name`,`desc`)
            VALUES(?,?,?,?,?);"#,
        uid,
        account_id,
        user_id,
        content.name,
        content.desc,
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
            r#"UPDATE `group` SET {}
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
    filter: &super::Querys,
) -> Result<UserGroupBindings> {
    let mut wheres = format!(r#"`id` = {}"#, id);
    if let Some(v) = &filter.account_id {
        let account_id_u64: u64 =
            v.parse().map_err(|err| Code::bad_request(&err))?;
        wheres.push_str(" AND ");
        wheres
            .push_str(format!(r#"`account_id` = {}"#, account_id_u64).as_str());
    }

    let row = match sqlx::query(
            format!(r#"SELECT `id`,`account_id`,`user_id`,`name`,`desc`,`created_at`,`updated_at`
            FROM `group`
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
    let mut usergroup_binding = UserGroupBindings {
        id: row.try_get::<u64, _>("id").map_err(Code::any)?.to_string(),
        account_id: row
            .try_get::<u64, _>("account_id")
            .map_err(Code::any)?
            .to_string(),
        user_id: row
            .try_get::<u64, _>("user_id")
            .map_err(Code::any)?
            .to_string(),
        name: row.try_get("name").map_err(Code::any)?,
        desc: row.try_get("desc").map_err(Code::any)?,
        links: Vec::new(),
        created_at: row.try_get("created_at").map_err(Code::any)?,
        updated_at: row.try_get("updated_at").map_err(Code::any)?,
    };
    let f = format!(
        "`id` = {} AND `deleted` = 0 {}",
        id,
        filter.pagination.to_string()
    );
    usergroup_binding
        .links
        .append(&mut list_user_group_role(pool, f.as_str()).await?);
    usergroup_binding
        .links
        .append(&mut list_user_group_user(pool, f.as_str()).await?);
    Ok(usergroup_binding)
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
            r#"UPDATE `group` SET `deleted` = `id`,`deleted_at`= '{}'
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
) -> Result<List<UserGroup>> {
    let mut wheres = String::from("");
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
            r#"SELECT COUNT(*) as count FROM `group`
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
                r#"SELECT `id`,`account_id`,`user_id`,`name`,`desc`,`created_at`,`updated_at`
                FROM `group`
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
        result.data.push(UserGroup {
            id: row.try_get::<u64, _>("id").map_err(Code::any)?.to_string(),
            account_id: row
                .try_get::<u64, _>("account_id")
                .map_err(Code::any)?
                .to_string(),
            user_id: row
                .try_get::<u64, _>("user_id")
                .map_err(Code::any)?
                .to_string(),
            name: row.try_get("name").map_err(Code::any)?,
            desc: row.try_get("desc").map_err(Code::any)?,
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
            r#"SELECT COUNT(*) as count FROM `group`
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
pub async fn add_user(
    pool: &MySqlPool,
    id: &str,
    account_id: &str,
    user_id: &str,
) -> Result<()> {
    if sqlx::query!(
        r#"SELECT `id` FROM `group`
            WHERE `id` = ? AND `account_id` = ? AND `deleted` = 0 LIMIT 1;"#,
        id,
        account_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(Code::any)?
    .is_none()
    {
        return Err(Code::not_found(&format!("not found user_group {}", id,)));
    }
    if sqlx::query!(
        r#"SELECT `id` FROM `user`
            WHERE `id` = ? AND `account_id` = ? AND `deleted` = 0 LIMIT 1;"#,
        user_id,
        account_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(Code::any)?
    .is_none()
    {
        return Err(Code::not_found(&format!("not found user {}", user_id)));
    }
    sqlx::query!(
        r#"INSERT INTO `group_user`
            (`id`,`user_group_id`,`user_id`)
            VALUES(?,?,?);"#,
        next_id().map_err(Code::any)?,
        id,
        user_id,
    )
    .execute(pool)
    .await
    .map_err(Code::any)?;
    Ok(())
}
pub async fn delete_user(
    pool: &MySqlPool,
    id: &str,
    user_id: &str,
) -> Result<()> {
    sqlx::query!(
        r#"UPDATE `group_user` SET `deleted` = `id`,`deleted_at`= ?
            WHERE `user_group_id` = ? AND `user_id` = ? AND `deleted` = 0;"#,
        Utc::now().naive_utc(),
        id,
        user_id,
    )
    .execute(pool)
    .await
    .map_err(Code::any)?;
    Ok(())
}
pub async fn add_role(
    pool: &MySqlPool,
    id: &str,
    account_id: &str,
    role_id: &str,
) -> Result<()> {
    if sqlx::query!(
        r#"SELECT `id` FROM `group`
            WHERE `id` = ? AND `account_id` = ? AND `deleted` = 0 LIMIT 1;"#,
        id,
        account_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(Code::any)?
    .is_none()
    {
        return Err(Code::not_found(&format!("not found user_group {}", id,)));
    }
    if sqlx::query!(
        r#"SELECT `id` FROM `role`
            WHERE `id` = ? AND `account_id` = ? AND `deleted` = 0 LIMIT 1;"#,
        role_id,
        account_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(Code::any)?
    .is_none()
    {
        return Err(Code::not_found(&format!("not found role {}", role_id)));
    }
    sqlx::query!(
        r#"INSERT INTO `group_role`
            (`id`,`user_group_id`,`role_id`)
            VALUES(?,?,?);"#,
        next_id().map_err(Code::any)?,
        id,
        role_id,
    )
    .execute(pool)
    .await
    .map_err(Code::any)?;
    Ok(())
}

pub async fn delete_role(
    pool: &MySqlPool,
    id: &str,
    role_id: &str,
) -> Result<()> {
    sqlx::query!(
        r#"UPDATE `group_role` SET `deleted` = `id`,`deleted_at`= ?
            WHERE `user_group_id` = ? AND `role_id` = ? AND `deleted` = 0;"#,
        Utc::now().naive_utc(),
        id,
        role_id,
    )
    .execute(pool)
    .await
    .map_err(Code::any)?;
    Ok(())
}

async fn list_user_group_user(
    pool: &MySqlPool,
    wheres: &str,
) -> Result<Vec<UserGroupBinding>> {
    let rows = sqlx::query(
        format!(
            r#"SELECT `user_id`
                FROM `group_user`
                WHERE {};"#,
            wheres,
        )
        .as_str(),
    )
    .fetch_all(pool)
    .await
    .map_err(Code::any)?;
    let mut data = Vec::with_capacity(rows.len());
    for row in rows.iter() {
        data.push(UserGroupBinding {
            kind: Kind::User,
            subject_id: row.try_get("user_id").map_err(Code::any)?,
        })
    }
    Ok(data)
}
async fn list_user_group_role(
    pool: &MySqlPool,
    wheres: &str,
) -> Result<Vec<UserGroupBinding>> {
    let rows = sqlx::query(
        format!(
            r#"SELECT `role_id`
                FROM `group_role`
                WHERE {};"#,
            wheres,
        )
        .as_str(),
    )
    .fetch_all(pool)
    .await
    .map_err(Code::any)?;

    let mut data = Vec::with_capacity(rows.len());

    for row in rows.iter() {
        data.push(UserGroupBinding {
            kind: Kind::Role,
            subject_id: row.try_get("role_id").map_err(Code::any)?,
        })
    }
    Ok(data)
}
