use async_trait::async_trait;
use serde_json::value::RawValue;
use sqlx::{types::Json, MySqlPool, Row};

use cim_slo::{errors, Result};

use crate::{authrequest::AuthRequest, Claim, Interface, List};

#[derive(Clone, Debug)]
pub struct AuthRequestImpl {
    pool: MySqlPool,
}

impl AuthRequestImpl {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Interface for AuthRequestImpl {
    type T = AuthRequest;
    type L = ();

    #[tracing::instrument]
    async fn put(&self, content: &Self::T) -> Result<()> {
        let client_id = content
            .client_id
            .parse::<u64>()
            .map_err(|err| errors::bad_request(&err))?;
        sqlx::query(
            r#"REPLACE INTO `auth_request`
            (`id`,`client_id`,`response_types`,`scopes`,`redirect_uri`,`code_challenge`,`code_challenge_method`,
             `nonce`,`state`,`claim`,`connector_id`,`connector_data`,`expiry`)
            VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?);"#,
        )
        .bind(&content.id)
        .bind(client_id)
        .bind(Json(&content.response_types))
        .bind(Json(&content.scopes))
        .bind(&content.redirect_uri)
        .bind(&content.code_challenge)
        .bind(&content.code_challenge_method)
        .bind(&content.nonce)
        .bind(&content.state)
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
        sqlx::query(r#"DELETE FROM `auth_request` WHERE id = ?;"#)
            .bind(&input.id)
            .execute(&self.pool)
            .await
            .map_err(errors::any)?;
        Ok(())
    }

    #[tracing::instrument]
    async fn get(&self, output: &mut Self::T) -> Result<()> {
        let row = match sqlx::query(
            r#"SELECT `id`,`client_id`,`response_types`,`scopes`,`redirect_uri`,`code_challenge`,`code_challenge_method`,
            `nonce`,`state`,`claim`,`connector_id`,`connector_data`,`expiry`,
            `created_at`,`updated_at`
            FROM `auth_request`
            WHERE id = ?;"#)
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
        output.client_id = row
            .try_get::<u64, _>("client_id")
            .map_err(errors::any)?
            .to_string();
        output.response_types = row
            .try_get::<Json<Vec<String>>, _>("response_types")
            .map_err(errors::any)?
            .0;
        output.scopes = row
            .try_get::<Json<Vec<String>>, _>("scopes")
            .map_err(errors::any)?
            .0;
        output.redirect_uri =
            row.try_get("redirect_uri").map_err(errors::any)?;
        output.code_challenge =
            row.try_get("code_challenge").map_err(errors::any)?;
        output.code_challenge_method =
            row.try_get("code_challenge_method").map_err(errors::any)?;
        output.nonce = row.try_get("nonce").map_err(errors::any)?;
        output.state = row.try_get("state").map_err(errors::any)?;
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
