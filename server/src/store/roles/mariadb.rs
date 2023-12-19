use chrono::Utc;
use sqlx::{MySqlPool, Row};

use crate::{errors, next_id, Result};

use crate::models::{
    role::{Kind, RoleBinding, RoleBindings},
    List, ID,
};

pub async fn create(
    pool: &MySqlPool,
    id: Option<String>,
    content: &super::Content,
) -> Result<ID> {
    let uid = match id {
        Some(v) => v.parse().map_err(|err| errors::bad_request(&err))?,
        None => next_id().map_err(errors::any)?,
    };
    let account_id: u64 = content
        .account_id
        .parse()
        .map_err(|err| errors::bad_request(&err))?;
    let user_id: u64 = content
        .user_id
        .parse()
        .map_err(|err| errors::bad_request(&err))?;
    sqlx::query!(
        r#"INSERT INTO `role`
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
    .map_err(errors::any)?;

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
            v.parse().map_err(|err| errors::bad_request(&err))?;
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
            r#"UPDATE `role` SET {}
                WHERE {};"#,
            update_content, wheres,
        )
        .as_str(),
    )
    .execute(pool)
    .await
    .map_err(errors::any)?;
    Ok(())
}

pub async fn get(
    pool: &MySqlPool,
    id: &str,
    filter: &super::Querys,
) -> Result<RoleBindings> {
    let mut wheres = format!(r#"`id` = {}"#, id);
    if let Some(v) = &filter.account_id {
        let account_id_u64: u64 =
            v.parse().map_err(|err| errors::bad_request(&err))?;
        wheres.push_str(" AND ");
        wheres
            .push_str(format!(r#"`account_id` = {}"#, account_id_u64).as_str());
    }

    let row = match sqlx::query(
            format!(r#"SELECT `id`,`account_id`,`user_id`,`name`,`desc`,`created_at`,`updated_at`
            FROM `role`
            WHERE {} AND `deleted` = 0;"#,
            wheres)
            .as_str()
        )
        .fetch_optional(pool)
        .await
        {
            Ok(v) => match v {
                Some(value) => Ok(value),
                None => Err(errors::not_found("no rows")),
            },
            Err(err) => Err(errors::any(err)),
        }?;
    let mut data = RoleBindings {
        id: row
            .try_get::<u64, _>("id")
            .map_err(errors::any)?
            .to_string(),
        account_id: row
            .try_get::<u64, _>("account_id")
            .map_err(errors::any)?
            .to_string(),
        user_id: row
            .try_get::<u64, _>("user_id")
            .map_err(errors::any)?
            .to_string(),
        name: row.try_get("name").map_err(errors::any)?,
        desc: row.try_get("desc").map_err(errors::any)?,
        links: Vec::new(),
        created_at: row.try_get("created_at").map_err(errors::any)?,
        updated_at: row.try_get("updated_at").map_err(errors::any)?,
    };
    let f = format!(
        "`role_id` = {} AND `deleted` = 0 {}",
        id,
        filter.pagination.to_string()
    );
    data.links
        .append(&mut list_role_policy(pool, f.as_str()).await?);
    data.links
        .append(&mut list_user_role(pool, f.as_str()).await?);
    Ok(data)
}

pub async fn delete(
    pool: &MySqlPool,
    id: &str,
    account_id: Option<String>,
) -> Result<()> {
    let mut wheres = format!(r#"`id` = {}"#, id);
    if let Some(v) = account_id {
        let account_id_u64: u64 =
            v.parse().map_err(|err| errors::bad_request(&err))?;
        wheres.push_str(" AND ");
        wheres
            .push_str(format!(r#"`account_id` = {}"#, account_id_u64).as_str());
    }
    let mut tx = pool.begin().await.map_err(errors::any)?;
    sqlx::query(
        format!(
            r#"UPDATE `role` SET `deleted` = `id`,`deleted_at`= '{}'
            WHERE {} AND `deleted` = 0;"#,
            Utc::now().naive_utc(),
            wheres
        )
        .as_str(),
    )
    .execute(&mut *tx)
    .await
    .map_err(errors::any)?;
    sqlx::query(
        r#"UPDATE  `role_policy`  SET  `deleted` = `id`,`deleted_at` =  ?
            WHERE  `role_id`  =  ?  AND  `deleted`  = 0"#,
    )
    .bind(Utc::now())
    .bind(id)
    .execute(&mut *tx)
    .await
    .map_err(errors::any)?;
    sqlx::query(
        r#"UPDATE  `user_role`  SET  `deleted` = `id`,`deleted_at` =  ?
            WHERE  `role_id`  =  ?  AND  `deleted`  = 0"#,
    )
    .bind(Utc::now())
    .bind(id)
    .execute(&mut *tx)
    .await
    .map_err(errors::any)?;
    tx.commit().await.map_err(errors::any)?;
    Ok(())
}

pub async fn list(
    pool: &MySqlPool,
    filter: &super::Querys,
) -> Result<List<super::Role>> {
    let mut wheres = String::from("");
    if let Some(account_id) = &filter.account_id {
        let account_id_u64: u64 = account_id
            .parse()
            .map_err(|err| errors::bad_request(&err))?;
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
            r#"SELECT COUNT(*) as count FROM `role`
            WHERE {};"#,
            wheres,
        )
        .as_str(),
    )
    .fetch_one(pool)
    .await
    .map_err(errors::any)?;
    // 查询列表
    let query = filter.pagination.to_string();
    if !query.is_empty() {
        wheres.push_str(query.as_str());
    }
    let rows = sqlx::query(
            format!(
                r#"SELECT `id`,`account_id`,`user_id`,`name`,`desc`,`created_at`,`updated_at`
                FROM `role`
                WHERE {};"#,
                wheres,
            )
            .as_str(),
        )
        .fetch_all(pool)
        .await
        .map_err(errors::any)?;
    let mut result = List {
        data: Vec::new(),
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
            user_id: row
                .try_get::<u64, _>("user_id")
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

pub async fn exist(
    pool: &MySqlPool,
    id: &str,
    account_id: Option<String>,
    unscoped: bool,
) -> Result<bool> {
    let mut wheres = format!(r#"`id` = {}"#, id);
    if let Some(v) = account_id {
        let account_id_u64: u64 =
            v.parse().map_err(|err| errors::bad_request(&err))?;
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
            r#"SELECT COUNT(*) as count FROM `role`
            WHERE {} LIMIT 1;"#,
            wheres,
        )
        .as_str(),
    )
    .fetch_one(pool)
    .await
    .map_err(errors::any)?;
    let count: i64 = result.try_get("count").map_err(errors::any)?;
    Ok(count != 0)
}

pub async fn add_user(
    pool: &MySqlPool,
    id: &str,
    account_id: &str,
    user_id: &str,
) -> Result<()> {
    if sqlx::query!(
        r#"SELECT `id` FROM `role`
            WHERE `id` = ? AND `account_id` = ? AND `deleted` = 0 LIMIT 1;"#,
        id,
        account_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(errors::any)?
    .is_none()
    {
        return Err(errors::not_found(&format!("not found role {}", id,)));
    }
    if sqlx::query!(
        r#"SELECT `id` FROM `user`
            WHERE `id` = ? AND `account_id` = ? AND `deleted` = 0 LIMIT 1;"#,
        user_id,
        account_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(errors::any)?
    .is_none()
    {
        return Err(errors::not_found(&format!("not found user {}", user_id)));
    }
    sqlx::query!(
        r#"INSERT INTO `user_role`
            (`id`,`user_id`,`role_id`)
            VALUES(?,?,?);"#,
        next_id().map_err(errors::any)?,
        user_id,
        id,
    )
    .execute(pool)
    .await
    .map_err(errors::any)?;
    Ok(())
}
pub async fn delete_user(
    pool: &MySqlPool,
    id: &str,
    user_id: &str,
) -> Result<()> {
    sqlx::query!(
        r#"UPDATE `user_role` SET `deleted` = `id`,`deleted_at`= ?
            WHERE `user_id` = ? AND `role_id` = ? AND `deleted` = 0;"#,
        Utc::now().naive_utc(),
        user_id,
        id,
    )
    .execute(pool)
    .await
    .map_err(errors::any)?;
    Ok(())
}
pub async fn add_policy(
    pool: &MySqlPool,
    id: &str,
    account_id: &str,
    policy_id: &str,
) -> Result<()> {
    if sqlx::query!(
        r#"SELECT `id` FROM `role`
            WHERE `id` = ? AND `account_id` = ? AND `deleted` = 0 LIMIT 1;"#,
        id,
        account_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(errors::any)?
    .is_none()
    {
        return Err(errors::not_found(&format!("not found role {}", id,)));
    }
    if sqlx::query!(
            r#"SELECT `id` FROM `policy`
            WHERE `id` = ? AND `account_id` IN(0,?) AND `deleted` = 0 LIMIT 1;"#,
            policy_id,
            account_id,
        )
        .fetch_optional(pool)
        .await
        .map_err(errors::any)?
        .is_none()
        {
            return Err(errors::not_found(&format!(
                "not found policy {}",
                policy_id
            )));
        }
    sqlx::query!(
        r#"INSERT INTO `role_policy`
            (`id`,`role_id`,`policy_id`)
            VALUES(?,?,?);"#,
        next_id().map_err(errors::any)?,
        id,
        policy_id,
    )
    .execute(pool)
    .await
    .map_err(errors::any)?;
    Ok(())
}

pub async fn delete_policy(
    pool: &MySqlPool,
    id: &str,
    policy_id: &str,
) -> Result<()> {
    sqlx::query!(
        r#"UPDATE `role_policy` SET `deleted` = `id`,`deleted_at`= ?
            WHERE `role_id` = ? AND `policy_id` = ? AND `deleted` = 0;"#,
        Utc::now().naive_utc(),
        id,
        policy_id,
    )
    .execute(pool)
    .await
    .map_err(errors::any)?;
    Ok(())
}

async fn list_user_role(
    pool: &MySqlPool,
    wheres: &str,
) -> Result<Vec<RoleBinding>> {
    let rows = sqlx::query(
        format!(
            r#"SELECT `user_id`
                FROM `user_role`
                WHERE {};"#,
            wheres,
        )
        .as_str(),
    )
    .fetch_all(pool)
    .await
    .map_err(errors::any)?;

    let mut data = Vec::with_capacity(rows.len());
    for row in rows.iter() {
        data.push(RoleBinding {
            kind: Kind::User,
            subject_id: row.try_get("user_id").map_err(errors::any)?,
        })
    }
    Ok(data)
}
async fn list_role_policy(
    pool: &MySqlPool,
    wheres: &str,
) -> Result<Vec<RoleBinding>> {
    let rows = sqlx::query(
        format!(
            r#"SELECT `policy_id`
                FROM `role_policy`
                WHERE {};"#,
            wheres,
        )
        .as_str(),
    )
    .fetch_all(pool)
    .await
    .map_err(errors::any)?;

    let mut data = Vec::with_capacity(rows.len());
    for row in rows.iter() {
        data.push(RoleBinding {
            kind: Kind::Policy,
            subject_id: row.try_get("policy_id").map_err(errors::any)?,
        })
    }
    Ok(data)
}
