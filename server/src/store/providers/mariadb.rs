use sqlx::{MySqlPool, Row};

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
    match sqlx::query(r#"SELECT `id`,`secret`,`redirect_url`,`name`,`prompt`,`logo_url` FROM `provider` WHERE `id` = ? AND `deleted` = 0"#)
        .bind(id)
            .fetch_optional(pool)
            .await
            {
                Ok(v) => match v {
                    Some(value) => {
                        Ok(Provider {
                id: value.try_get::<u64, _>("id").map_err(errors::any)?.to_string(),
                secret: value.try_get("secret").map_err(errors::any)?,
                redirect_url: value.try_get("redirect_url").map_err(errors::any)?,
                name: value.try_get("name").map_err(errors::any)?,
                prompt:value.try_get("prompt").map_err(errors::any)?,
                logo_url: value.try_get("logo_url").map_err(errors::any)?,
                refresh:false,
            })},
                    None => Err(errors::not_found("no rows")),
                },
                Err(err) => Err(errors::any(err)),
            }
}

pub async fn list(pool: &MySqlPool) -> Result<Vec<Provider>> {
    let rows=sqlx::query(r#"SELECT `id`,`secret`,`redirect_url`,`name`,`prompt`,`logo_url` FROM `provider` WHERE `deleted` = 0"#)
            .fetch_all(pool)
            .await
            .map_err(errors::any)?;
    let mut datas = Vec::with_capacity(rows.len());
    for value in rows.iter() {
        datas.push(Provider {
            id: value
                .try_get::<u64, _>("id")
                .map_err(errors::any)?
                .to_string(),
            secret: value.try_get("secret").map_err(errors::any)?,
            redirect_url: value.try_get("redirect_url").map_err(errors::any)?,
            name: value.try_get("name").map_err(errors::any)?,
            prompt: value.try_get("prompt").map_err(errors::any)?,
            logo_url: value.try_get("logo_url").map_err(errors::any)?,
            refresh: false,
        })
    }
    Ok(datas)
}
