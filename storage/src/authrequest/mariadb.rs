use async_trait::async_trait;
use serde_json::value::RawValue;
use sqlx::{MySqlPool, Row};

use slo::{errors, next_id, Result};

use crate::ID;

use super::{AuthRequest, AuthRequestStore};

pub struct AuthRequestImpl {
    pool: MySqlPool,
}

impl AuthRequestImpl {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuthRequestStore for AuthRequestImpl {
    async fn put_auth_request(&self, content: &AuthRequest) -> Result<ID> {
        let id = if content.id.is_empty() {
            next_id().map_err(errors::any)?
        } else {
            content
                .id
                .parse::<u64>()
                .map_err(|err| errors::bad_request(&err))?
        };

        let response_types = serde_json::to_string(&content.response_types)
            .map_err(errors::any)?;

        let scopes =
            serde_json::to_string(&content.scopes).map_err(errors::any)?;

        let claim =
            serde_json::to_string(&content.claim).map_err(errors::any)?;

        sqlx::query(
            r#"REPLACE INTO `auth_request`
            (`id`,`client_id`,`response_types`,`scopes`,`redirect_uri`,`code_challenge`,`code_challenge_method`,
             `nonce`,`state`,`hmac_key`,`force_approval_prompt`,`logged_in`,`claim`,`connector_id`,`connector_data`,`expiry`)
            VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?);"#,
        )
        .bind(id)
        .bind(&content.client_id)
        .bind(response_types)
        .bind(scopes)
        .bind(&content.redirect_uri)
        .bind(&content.code_challenge)
        .bind(&content.code_challenge_method)
        .bind(&content.nonce)
        .bind(&content.state)
        .bind(&content.hmac_key)
        .bind(content.force_approval_prompt)
        .bind(content.logged_in)
        .bind(claim)
        .bind(&content.connector_id)
        .bind(content.connector_data.as_ref().map(|v| v.to_string()))
        .bind(content.expiry)
        .execute(&self.pool)
        .await
        .map_err(errors::any)?;
        Ok(ID { id: id.to_string() })
    }
    async fn get_auth_request(&self, id: &str) -> Result<AuthRequest> {
        let row = match sqlx::query(
            r#"SELECT `id`,`client_id`,`response_types`,`scopes`,`redirect_uri`,`code_challenge`,`code_challenge_method`,
            `nonce`,`state`,`hmac_key`,`force_approval_prompt`,`logged_in`,`claim`,`connector_id`,`connector_data`,`expiry`
            FROM `auth_request`
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

        let response_types = serde_json::from_str(
            &row.try_get::<String, _>("response_types")
                .map_err(errors::any)?,
        )
        .map_err(errors::any)?;

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

        Ok(AuthRequest {
            id: row
                .try_get::<u64, _>("id")
                .map_err(errors::any)?
                .to_string(),
            client_id: row.try_get("client_id").map_err(errors::any)?,
            response_types,
            scopes,
            redirect_uri: row.try_get("redirect_uri").map_err(errors::any)?,
            code_challenge: row
                .try_get("code_challenge")
                .map_err(errors::any)?,
            code_challenge_method: row
                .try_get("code_challenge_method")
                .map_err(errors::any)?,
            nonce: row.try_get("nonce").map_err(errors::any)?,
            state: row.try_get("state").map_err(errors::any)?,
            hmac_key: row.try_get("hmac_key").map_err(errors::any)?,
            force_approval_prompt: row
                .try_get("force_approval_prompt")
                .map_err(errors::any)?,
            logged_in: row.try_get("logged_in").map_err(errors::any)?,
            claim,
            connector_id: row.try_get("connector_id").map_err(errors::any)?,
            connector_data,
            expiry: row.try_get("expiry").map_err(errors::any)?,
            created_at: row.try_get("created_at").map_err(errors::any)?,
            updated_at: row.try_get("updated_at").map_err(errors::any)?,
        })
    }
    async fn delete_auth_request(&self, id: &str) -> Result<()> {
        sqlx::query(
            r#"UPDATE `auth_request` SET `deleted` = `id`,`deleted_at`= now()
            WHERE id = ? AND `deleted` = 0;"#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(errors::any)?;
        Ok(())
    }
}
