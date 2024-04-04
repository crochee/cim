use async_trait::async_trait;
use serde_json::value::RawValue;
use sqlx::{MySqlPool, Row};

use slo::{errors, next_id, Result};

use crate::ID;

use super::{AuthCode, AuthCodeStore};

pub struct AuthCodeImpl {
    pool: MySqlPool,
}

impl AuthCodeImpl {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuthCodeStore for AuthCodeImpl {
    async fn put_auth_code(&self, content: &AuthCode) -> Result<ID> {
        let id = if content.id.is_empty() {
            next_id().map_err(errors::any)?
        } else {
            content
                .id
                .parse::<u64>()
                .map_err(|err| errors::bad_request(&err))?
        };

        let scopes =
            serde_json::to_string(&content.scopes).map_err(errors::any)?;

        let claim =
            serde_json::to_string(&content.claim).map_err(errors::any)?;

        sqlx::query(
            r#"REPLACE INTO `auth_code`
            (`id`,`client_id`,`scopes`,`nonce`,`redirect_uri`,`code_challenge`,`code_challenge_method`,
            `claim`,`connector_id`,`connector_data`,`expiry`)
            VALUES(?,?,?,?,?,?,?,?,?,?,?);"#,
        )
        .bind(id)
        .bind(&content.client_id)
        .bind(scopes)
        .bind(&content.nonce)
        .bind(&content.redirect_uri)
        .bind(&content.code_challenge)
        .bind(&content.code_challenge_method)
        .bind(claim)
        .bind(&content.connector_id)
        .bind(content.connector_data.as_ref().map(|v| v.to_string()))
        .bind(content.expiry)
        .execute(&self.pool)
        .await
        .map_err(errors::any)?;
        Ok(ID { id: id.to_string() })
    }
    async fn get_auth_code(&self, id: &str) -> Result<AuthCode> {
        let row = match sqlx::query(
            r#"SELECT `id`,`client_id`,`scopes`,`nonce`,`state`,`redirect_uri`,`code_challenge`,`code_challenge_method`,
            `claim`,`connector_id`,`connector_data`,`expiry`
            FROM `auth_code`
            WHERE id = ? AND deleted = 0;"#)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        {
            Ok(v) => match v {
                Some(value) => Ok(value),
                None => Err(errors::not_found("no rows")),
            },
            Err(err) => Err(errors::any(err)),
        }?;

        let scopes = serde_json::from_str(
            &row.try_get::<String, _>("scopes").map_err(errors::any)?,
        )
        .map_err(errors::any)?;

        let claim = serde_json::from_str(
            &row.try_get::<String, _>("claim").map_err(errors::any)?,
        )
        .map_err(errors::any)?;

        let connector_data = row
            .try_get::<Option<String>, _>("connector_data")
            .map_err(errors::any)?
            .map(|v| RawValue::from_string(v).unwrap());

        Ok(AuthCode {
            id: row
                .try_get::<u64, _>("id")
                .map_err(errors::any)?
                .to_string(),
            client_id: row.try_get("client_id").map_err(errors::any)?,
            scopes,
            nonce: row.try_get("nonce").map_err(errors::any)?,
            redirect_uri: row.try_get("redirect_uri").map_err(errors::any)?,
            code_challenge: row
                .try_get("code_challenge")
                .map_err(errors::any)?,
            code_challenge_method: row
                .try_get("code_challenge_method")
                .map_err(errors::any)?,
            claim,
            connector_id: row.try_get("connector_id").map_err(errors::any)?,
            connector_data,
            expiry: row.try_get("expiry").map_err(errors::any)?,
            created_at: row.try_get("created_at").map_err(errors::any)?,
            updated_at: row.try_get("updated_at").map_err(errors::any)?,
        })
    }
    async fn delete_auth_code(&self, id: &str) -> Result<()> {
        sqlx::query(
            r#"UPDATE `auth_code` SET `deleted` = `id`,`deleted_at`= now()
            WHERE id = ? AND `deleted` = 0;"#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(errors::any)?;
        Ok(())
    }
}
