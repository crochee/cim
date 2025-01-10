use async_trait::async_trait;
use serde_json::value::RawValue;
use sqlx::{types::Json, MySqlPool, Row};

use cim_slo::{errors, Result};

use crate::{authcode::AuthCode, Claim, Interface, List};

#[derive(Clone, Debug)]
pub struct AuthCodeImpl {
    pool: MySqlPool,
}

impl AuthCodeImpl {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Interface for AuthCodeImpl {
    type T = AuthCode;
    type L = ();

    #[tracing::instrument]
    async fn put(&self, content: &Self::T) -> Result<()> {
        sqlx::query(
            r#"REPLACE INTO `auth_code`
            (`id`,`client_id`,`scopes`,`nonce`,`redirect_uri`,`code_challenge`,`code_challenge_method`,
            `claim`,`connector_id`,`connector_data`,`expiry`)
            VALUES(?,?,?,?,?,?,?,?,?,?,?);"#,
        )
        .bind(&content.id)
        .bind(&content.client_id)
        .bind(Json(&content.scopes))
        .bind(&content.nonce)
        .bind(&content.redirect_uri)
        .bind(&content.code_challenge)
        .bind(&content.code_challenge_method)
        .bind(Json(&content.claim))
        .bind(&content.connector_id)
        .bind(content.connector_data.as_ref().map(|v| v.to_string()))
        .bind(content.expiry)
        .execute(&self.pool)
        .await
        .map_err(errors::any)?;

        Ok(())
    }

    #[tracing::instrument]
    async fn delete(&self, input: &Self::T) -> Result<()> {
        sqlx::query(
            r#"UPDATE `auth_code` SET `deleted` = `id`,`deleted_at`= now()
            WHERE id = ? AND `deleted` = 0;"#,
        )
        .bind(&input.id)
        .execute(&self.pool)
        .await
        .map_err(errors::any)?;
        Ok(())
    }

    #[tracing::instrument]
    async fn get(&self, output: &mut Self::T) -> Result<()> {
        let row = match sqlx::query(
            r#"SELECT `id`,`client_id`,`scopes`,`nonce`,`state`,`redirect_uri`,`code_challenge`,`code_challenge_method`,
            `claim`,`connector_id`,`connector_data`,`expiry`
            FROM `auth_code`
            WHERE id = ? AND deleted = 0;"#)
        .bind(&output.id)
        .fetch_optional(&self.pool)
        .await
        {
            Ok(v) => match v {
                Some(value) => Ok(value),
                None => Err(errors::not_found("no rows")),
            },
            Err(err) => Err(errors::any(err)),
        }?;

        output.id = row.try_get("id").map_err(errors::any)?;
        output.client_id = row.try_get("client_id").map_err(errors::any)?;
        output.scopes = row
            .try_get::<Json<Vec<String>>, _>("scopes")
            .map_err(errors::any)?
            .0;
        output.nonce = row.try_get("nonce").map_err(errors::any)?;
        output.redirect_uri =
            row.try_get("redirect_uri").map_err(errors::any)?;
        output.code_challenge =
            row.try_get("code_challenge").map_err(errors::any)?;
        output.code_challenge_method =
            row.try_get("code_challenge_method").map_err(errors::any)?;
        output.claim = row
            .try_get::<Json<Claim>, _>("claim")
            .map_err(errors::any)?
            .0;
        output.connector_id =
            row.try_get("connector_id").map_err(errors::any)?;
        output.connector_data = row
            .try_get::<Option<String>, _>("connector_data")
            .map_err(errors::any)?
            .map(|v| RawValue::from_string(v).unwrap());
        output.expiry = row.try_get("expiry").map_err(errors::any)?;
        output.created_at = row.try_get("created_at").map_err(errors::any)?;
        output.updated_at = row.try_get("updated_at").map_err(errors::any)?;

        Ok(())
    }
    async fn list(
        &self,
        _opts: &Self::L,
        _output: &mut List<Self::T>,
    ) -> Result<()> {
        todo!()
    }

    async fn count(&self, _opts: &Self::L, _unscoped: bool) -> Result<i64> {
        todo!()
    }
}
