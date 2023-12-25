use chrono::Utc;
use rand::Rng;
use sqlx::{MySqlPool, Row};

use crate::{errors, next_id, Result};

use crate::models::{auth_request::AuthRequest, ID};

use super::{Content, UpdateOpts};

pub async fn create(pool: &MySqlPool, content: &Content) -> Result<ID> {
    let uid = next_id().map_err(errors::any)?;
    let response_type =
        serde_json::to_string(&content.response_types).map_err(errors::any)?;
    let scope = serde_json::to_string(&content.scopes).map_err(errors::any)?;
    let hmac_key = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(255)
        .map(char::from)
        .collect::<String>();
    sqlx::query(
            r#"INSERT INTO `auth_request`
            (`id`,`client_id`,`response_type`,`scope`,`redirect_url`,`nonce`,`state`,`force_approval`,`expiry`,`logged_in`,`hmac_key`)
            VALUES(?,?,?,?,?,?,?,?,?,?,?);"#,
            )
           .bind( uid)
           .bind( &content.client_id)
           .bind( response_type)
           .bind( scope)
           .bind( &content.redirect_url)
           .bind( &content.nonce)
           .bind( &content.state)
           .bind( content.force_approval)
           .bind( content.expiry)
           .bind( content.logged_in)
           .bind( hmac_key)
        .execute(pool)
        .await
        .map_err(errors::any)?;
    Ok(ID {
        id: uid.to_string(),
    })
}

pub async fn get(pool: &MySqlPool, id: &str) -> Result<AuthRequest> {
    let row  = match  sqlx::query(r#"SELECT `id`,`client_id`,`response_type`,`scope`,`redirect_url`,`nonce`,`state`,`force_approval`,`expiry`,`logged_in`,`claims`,`hmac_key`
        FROM `auth_request` 
        WHERE `id` = ? AND `deleted` = 0"#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        {
            Ok(v) => match v {
                Some(value) => Ok(value),
                None => Err(errors::not_found("no rows")),
            },
            Err(err) => Err(errors::any(err)),
        }?;
    Ok(AuthRequest {
        id: row
            .try_get::<u64, _>("id")
            .map_err(errors::any)?
            .to_string(),
        client_id: row.try_get("client_id").map_err(errors::any)?,
        response_types: serde_json::from_str(
            row.try_get("response_type").map_err(errors::any)?,
        )
        .map_err(errors::any)?,
        scopes: serde_json::from_str(
            row.try_get("scope").map_err(errors::any)?,
        )
        .map_err(errors::any)?,
        redirect_url: row.try_get("redirect_url").map_err(errors::any)?,
        nonce: row.try_get("nonce").map_err(errors::any)?,
        state: row.try_get("state").map_err(errors::any)?,
        force_approval: row
            .try_get::<u64, _>("force_approval")
            .map_err(errors::any)?
            == 0,
        expiry: row.try_get("expiry").map_err(errors::any)?,
        logged_in: row.try_get::<u64, _>("logged_in").map_err(errors::any)?
            == 0,
        claims: match row
            .try_get::<Option<String>, _>("claims")
            .map_err(errors::any)?
        {
            Some(v) => serde_json::from_str(&v).map_err(errors::any)?,
            None => None,
        },
        hmac_key: row.try_get("hmac_key").map_err(errors::any)?,
    })
}

pub async fn update(
    pool: &MySqlPool,
    id: &str,
    opts: &UpdateOpts,
) -> Result<()> {
    let claims = serde_json::to_string(&opts.claims).map_err(errors::any)?;
    sqlx::query(
        r#"UPDATE `auth_request` SET `logged_in` = ?,`claims`= ?
        WHERE `id` = ? AND `deleted` = 0;"#,
    )
    .bind(opts.logged_in)
    .bind(claims)
    .bind(id)
    .execute(pool)
    .await
    .map_err(errors::any)?;
    Ok(())
}

pub async fn delete(pool: &MySqlPool, id: &str) -> Result<()> {
    sqlx::query(
        r#"UPDATE `auth_request` SET `deleted` = `id`,`deleted_at`= ?
        WHERE `id` = ? AND `deleted` = 0;"#,
    )
    .bind(Utc::now().naive_utc())
    .bind(id)
    .execute(pool)
    .await
    .map_err(errors::any)?;
    Ok(())
}
