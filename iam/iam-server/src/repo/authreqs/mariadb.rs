use async_trait::async_trait;

use chrono::Utc;
use rand::Rng;
use sqlx::MySqlPool;

use cim_core::{next_id, Error, Result};

use crate::models::{auth_request::AuthRequest, ID};

use super::{AuthReqsRep, Content, UpdateOpts};

#[derive(Clone)]
pub struct MariadbAuthReqs {
    pool: MySqlPool,
}

impl MariadbAuthReqs {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuthReqsRep for MariadbAuthReqs {
    async fn create(&self, content: &Content) -> Result<ID> {
        let uid = next_id().map_err(Error::any)?;
        let response_type = serde_json::to_string(&content.response_types)
            .map_err(Error::any)?;
        let scope =
            serde_json::to_string(&content.scopes).map_err(Error::any)?;
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
        .execute(&self.pool)
        .await
        .map_err(Error::any)?;
        Ok(ID {
            id: uid.to_string(),
        })
    }
    async fn get(&self, id: &str) -> Result<AuthRequest> {
        let row=match sqlx::query!(r#"SELECT `id`,`client_id`,`response_type`,`scope`,`redirect_url`,`nonce`,`state`,`force_approval`,`expiry`,`logged_in`,`claims`,`hmac_key` 
        FROM `auth_request` 
        WHERE `id` = ? AND `deleted` = 0"#,
        id)
        .fetch_optional(&self.pool)
        .await
        {
            Ok(v) => match v {
                Some(value) => Ok(value),
                None => Err(Error::NotFound("no rows".to_owned())),
            },
            Err(err) => Err(Error::any(err)),
        }?;
        Ok(AuthRequest {
            id: row.id.to_string(),
            client_id: row.client_id,
            response_types: serde_json::from_str(&row.response_type)
                .map_err(Error::any)?,
            scopes: serde_json::from_str(&row.scope).map_err(Error::any)?,
            redirect_url: row.redirect_url,
            nonce: row.nonce,
            state: row.state,
            force_approval: row.force_approval == 0,
            expiry: row.expiry,
            logged_in: row.logged_in == 0,
            claims: match row.claims {
                Some(v) => serde_json::from_str(&v).map_err(Error::any)?,
                None => None,
            },
            hmac_key: row.hmac_key,
        })
    }
    async fn update(&self, id: &str, opts: &UpdateOpts) -> Result<()> {
        let claims = serde_json::to_string(&opts.claims).map_err(Error::any)?;
        sqlx::query!(
            r#"UPDATE `auth_request` SET `logged_in` = ?,`claims`= ?
        WHERE `id` = ? AND `deleted` = 0;"#,
            opts.logged_in,
            claims,
            id,
        )
        .execute(&self.pool)
        .await
        .map_err(Error::any)?;
        Ok(())
    }
    async fn delete(&self, id: &str) -> Result<()> {
        sqlx::query!(
            r#"UPDATE `auth_request` SET `deleted` = `id`,`deleted_at`= ?
        WHERE `id` = ? AND `deleted` = 0;"#,
            Utc::now().naive_utc(),
            id,
        )
        .execute(&self.pool)
        .await
        .map_err(Error::any)?;
        Ok(())
    }
}
