use chrono::Utc;
use sqlx::{MySqlPool, Row};

use cim_core::{next_id, Code, Result};

use crate::models::{
    policy::{Policy, Statement},
    List, ID,
};

pub async fn create(
    pool: &MySqlPool,
    id: Option<String>,
    content: &super::Content,
) -> Result<ID> {
    let account_id: u64 = content
        .account_id
        .clone()
        .unwrap_or_else(|| "0".to_owned())
        .parse()
        .map_err(|err| Code::bad_request(&err))?;

    let mut user_id: u64 = 0;
    if account_id > 0 {
        user_id = content
            .user_id
            .clone()
            .unwrap_or_else(|| "0".to_owned())
            .parse()
            .map_err(|err| Code::bad_request(&err))?;
    };

    let uid = match id {
        Some(v) => v.parse().map_err(|err| Code::bad_request(&err))?,
        None => next_id().map_err(Code::any)?,
    };

    let statement =
        serde_json::to_string(&content.statement).map_err(Code::any)?;

    sqlx::query!(
        r#"INSERT INTO `policy`
            (`id`,`account_id`,`user_id`,`desc`,`version`,`content`)
            VALUES(?,?,?,?,?,?);"#,
        uid,
        account_id,
        user_id,
        content.desc,
        content.version,
        statement,
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
    if let Some(desc) = &opts.desc {
        update_content.push_str(format!(r#"`desc` = '{}'"#, desc).as_str());
    };
    if let Some(version) = &opts.version {
        if !update_content.is_empty() {
            update_content.push_str(" , ");
        }
        update_content
            .push_str(format!(r#"`version` = '{}'"#, version).as_str());
    };
    if let Some(statement) = &opts.statement {
        if !update_content.is_empty() {
            update_content.push_str(" , ");
        }
        let content = serde_json::to_string(statement).map_err(Code::any)?;
        update_content
            .push_str(format!(r#"`content` = '{}'"#, content).as_str());
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
            r#"UPDATE `policy` SET {}
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
) -> Result<Policy> {
    let mut wheres = format!(r#"`id` = {}"#, id);
    if let Some(v) = account_id {
        let account_id_u64: u64 =
            v.parse().map_err(|err| Code::bad_request(&err))?;
        wheres.push_str(" AND ");
        wheres
            .push_str(format!(r#"`account_id` = {}"#, account_id_u64).as_str());
    }

    let row = match sqlx::query(
            format!(r#"SELECT `id`,`account_id`,`user_id`,`desc`,`version`,`content`,`created_at`,`updated_at`
            FROM `policy`
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

    let v = row.try_get::<u64, _>("account_id").map_err(Code::any)?;
    let account_id_str = if v == 0 { None } else { Some(v.to_string()) };
    let v = row.try_get::<u64, _>("user_id").map_err(Code::any)?;
    let user_id_str = if v == 0 { None } else { Some(v.to_string()) };
    let v = row.try_get::<String, _>("content").map_err(Code::any)?;
    let statement: Vec<Statement> =
        serde_json::from_str(&v).map_err(Code::any)?;
    Ok(Policy {
        id: row.try_get::<u64, _>("id").map_err(Code::any)?.to_string(),
        account_id: account_id_str,
        user_id: user_id_str,
        desc: row.try_get("desc").map_err(Code::any)?,
        version: row.try_get("version").map_err(Code::any)?,
        statement,
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
            r#"UPDATE `policy` SET `deleted` = `id`,`deleted_at`= '{}'
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
) -> Result<List<Policy>> {
    let mut wheres = String::from("");
    if let Some(version) = &filter.version {
        wheres.push_str(format!(r#"`version` = {}"#, version).as_str());
    };
    if let Some(account_id) = &filter.account_id {
        let account_id_u64: u64 =
            account_id.parse().map_err(|err| Code::bad_request(&err))?;
        if !wheres.is_empty() {
            wheres.push_str(" AND ");
        }
        wheres.push_str(
            format!(r#"`account_id` IN (0,{})"#, account_id_u64).as_str(),
        );
    };
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
                r#"SELECT `id`,`account_id`,`user_id`,`desc`,`version`,`content`,`created_at`,`updated_at`
                FROM `policy`
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
        let v = row.try_get::<u64, _>("account_id").map_err(Code::any)?;
        let account_id_str = if v == 0 { None } else { Some(v.to_string()) };
        let v = row.try_get::<u64, _>("user_id").map_err(Code::any)?;
        let user_id_str = if v == 0 { None } else { Some(v.to_string()) };
        let v = row.try_get::<String, _>("content").map_err(Code::any)?;
        let statement: Vec<Statement> =
            serde_json::from_str(&v).map_err(Code::any)?;

        result.data.push(Policy {
            id: row.try_get::<u64, _>("id").map_err(Code::any)?.to_string(),
            account_id: account_id_str,
            user_id: user_id_str,
            desc: row.try_get("desc").map_err(Code::any)?,
            version: row.try_get("version").map_err(Code::any)?,
            statement,
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
            r#"SELECT COUNT(*) as count FROM `policy`
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

pub async fn get_by_user(
    pool: &MySqlPool,
    user_id_str: &str,
) -> Result<Vec<Policy>> {
    let user_id: u64 = user_id_str.parse().map_err(Code::any)?;
    let sql = format!(
        r#"SELECT t3.`id`,t3.`account_id`,t3.`user_id`,t3.`desc`,t3.`version`,t3.`content`,t3.`created_at`,t3.`updated_at`
            FROM ((SELECT a3.`id` FROM `user` a1 RIGHT JOIN `user_role` a2 ON a1.`id`=a2.`user_id` RIGHT JOIN `role` a3 ON a2.`role_id`=a3.`id` WHERE a1.`id`= {} AND a1.`deleted`=0 AND a2.`deleted`=0 AND a3.`deleted`=0)
            UNION 
            (SELECT b4.`id` FROM `user` b1 RIGHT JOIN `user_group_user` b2 ON b1.`id`=b2.`user_id` RIGHT JOIN `user_group_role` b3 ON b2.`user_group_id`=b3.`user_group_id` RIGHT JOIN `role` b4 ON b3.`role_id`=b4.`id` 
            WHERE b1.`id`= {} AND b1.`deleted`=0 AND b2.`deleted`=0 AND b3.`deleted`=0 AND b4.`deleted`=0))
            t1 RIGHT JOIN `role_policy` t2 ON t1.`id`=t2.`role_id` RIGHT JOIN `policy` t3 ON t2.`policy_id`=t3.`id` WHERE t2.`deleted`=0 AND t3.`deleted`=0;"#,
        user_id, user_id,
    );
    let rows = sqlx::query(sql.as_str())
        .fetch_all(pool)
        .await
        .map_err(Code::any)?;

    let mut result = Vec::with_capacity(rows.len());

    for row in rows.iter() {
        let v = row.try_get::<u64, _>("account_id").map_err(Code::any)?;
        let account_id_str = if v == 0 { None } else { Some(v.to_string()) };
        let v = row.try_get::<u64, _>("user_id").map_err(Code::any)?;
        let user_id_str = if v == 0 { None } else { Some(v.to_string()) };
        let v = row.try_get::<String, _>("content").map_err(Code::any)?;
        let statement: Vec<Statement> =
            serde_json::from_str(&v).map_err(Code::any)?;

        result.push(Policy {
            id: row.try_get::<u64, _>("id").map_err(Code::any)?.to_string(),
            account_id: account_id_str,
            user_id: user_id_str,
            desc: row.try_get("desc").map_err(Code::any)?,
            version: row.try_get("version").map_err(Code::any)?,
            statement,
            created_at: row.try_get("created_at").map_err(Code::any)?,
            updated_at: row.try_get("updated_at").map_err(Code::any)?,
        })
    }
    Ok(result)
}
