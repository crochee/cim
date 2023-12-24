use sqlx::MySqlPool;

use crate::models::{provider::Provider, ID};
use crate::{errors, next_id, Result};

pub async fn create(pool: &MySqlPool, content: &super::Content) -> Result<ID> {
    let uid = next_id().map_err(errors::any)?;
    sqlx::query(
        r#"INSERT INTO `provider`
            (`id`,`secret`,`redirect_url`,`name`,`prompt`,`logo_url`)
            VALUES(?,?,?,?,?,?);"#,
    )
    .bind(uid)
    .bind(&content.secret)
    .bind(&content.redirect_url)
    .bind(&content.name)
    .bind(&content.prompt)
    .bind(&content.logo_url)
    .execute(pool)
    .await
    .map_err(errors::any)?;

    Ok(ID {
        id: uid.to_string(),
    })
}

pub async fn get(pool: &MySqlPool, id: &str) -> Result<Provider> {
    match sqlx::query!(r#"SELECT `id`,`secret`,`redirect_url`,`name`,`prompt`,`logo_url` FROM `provider` WHERE `id` = ? AND `deleted` = 0"#,id)
            .map(|row| Provider {
                id: row.id.to_string(),
                secret: row.secret,
                redirect_url: row.redirect_url,
                name: row.name,
                prompt:row.prompt,
                logo_url: row.logo_url,
                refresh:false,
            })
            .fetch_optional(pool)
            .await
            {
                Ok(v) => match v {
                    Some(value) => Ok(value),
                    None => Err(errors::not_found("no rows")),
                },
                Err(err) => Err(errors::any(err)),
            }
}

pub async fn list(pool: &MySqlPool) -> Result<Vec<Provider>> {
    sqlx::query!(r#"SELECT `id`,`secret`,`redirect_url`,`name`,`prompt`,`logo_url` FROM `provider` WHERE `deleted` = 0"#)
            .map(|row| Provider {
                id: row.id.to_string(),
                secret: row.secret,
                redirect_url: row.redirect_url,
                name: row.name,
                prompt:row.prompt,
                logo_url: row.logo_url,
                refresh:false,
            })
            .fetch_all(pool)
            .await
            .map_err(errors::any)
}
