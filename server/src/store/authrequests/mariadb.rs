use chrono::Utc;
use rand::Rng;
use sqlx::MySqlPool;

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
    sqlx::query!(
            r#"INSERT INTO `auth_request`
            (`id`,`client_id`,`response_type`,`scope`,`redirect_url`,`nonce`,`state`,`force_approval`,`expiry`,`logged_in`,`hmac_key`)
            VALUES(?,?,?,?,?,?,?,?,?,?,?);"#,
            uid,
            content.client_id,
            response_type,
            scope,
            content.redirect_url,
            content.nonce,
            content.state,
            content.force_approval,
            content.expiry,
            content.logged_in,
            hmac_key,
        )
        .execute(pool)
        .await
        .map_err(errors::any)?;
    Ok(ID {
        id: uid.to_string(),
    })
}

pub async fn get(pool: &MySqlPool, id: &str) -> Result<AuthRequest> {
    let row=match sqlx::query!(r#"SELECT `id`,`client_id`,`response_type`,`scope`,`redirect_url`,`nonce`,`state`,`force_approval`,`expiry`,`logged_in`,`claims`,`hmac_key` 
        FROM `auth_request` 
        WHERE `id` = ? AND `deleted` = 0"#,
        id)
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
        id: row.id.to_string(),
        client_id: row.client_id,
        response_types: serde_json::from_str(&row.response_type)
            .map_err(errors::any)?,
        scopes: serde_json::from_str(&row.scope).map_err(errors::any)?,
        redirect_url: row.redirect_url,
        nonce: row.nonce,
        state: row.state,
        force_approval: row.force_approval == 0,
        expiry: row.expiry,
        logged_in: row.logged_in == 0,
        claims: match row.claims {
            Some(v) => serde_json::from_str(&v).map_err(errors::any)?,
            None => None,
        },
        hmac_key: row.hmac_key,
    })
}

pub async fn update(
    pool: &MySqlPool,
    id: &str,
    opts: &UpdateOpts,
) -> Result<()> {
    let claims = serde_json::to_string(&opts.claims).map_err(errors::any)?;
    sqlx::query!(
        r#"UPDATE `auth_request` SET `logged_in` = ?,`claims`= ?
        WHERE `id` = ? AND `deleted` = 0;"#,
        opts.logged_in,
        claims,
        id,
    )
    .execute(pool)
    .await
    .map_err(errors::any)?;
    Ok(())
}

pub async fn delete(pool: &MySqlPool, id: &str) -> Result<()> {
    sqlx::query!(
        r#"UPDATE `auth_request` SET `deleted` = `id`,`deleted_at`= ?
        WHERE `id` = ? AND `deleted` = 0;"#,
        Utc::now().naive_utc(),
        id,
    )
    .execute(pool)
    .await
    .map_err(errors::any)?;
    Ok(())
}
