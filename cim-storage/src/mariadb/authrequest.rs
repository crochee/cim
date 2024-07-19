use async_trait::async_trait;
use sqlx::{MySqlPool, Row};
use serde_json::value::RawValue;

use cim_slo::{errors, Result};
use cim_watch::Watcher;

use crate::{authrequest::AuthRequest, Event, Interface, List};

#[derive(Clone)]
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
    async fn put(&self, content: &Self::T, _ttl: u64) -> Result<()> {

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
        .bind(&content.id)
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


       Ok(())
    }

    async fn delete(&self, id: &str) -> Result<()> {
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
    async fn get(&self, id: &str, output: &mut Self::T) -> Result<()> {
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



         output.   id= row .try_get::<u64, _>("id")
         .map_err(errors::any)?
         .to_string();
         output.   client_id= row.try_get("client_id").map_err(errors::any)?;
         output.   response_types= serde_json::from_str(
            &row.try_get::<String, _>("response_types")
                .map_err(errors::any)?,
        )
        .map_err(errors::any)?;
         output.   scopes=serde_json::from_str(
            &row.try_get::<String, _>("scopes").map_err(errors::any)?,
        )
        .map_err(errors::any)?;
         output.   redirect_uri= row.try_get("redirect_uri").map_err(errors::any)?;
         output.   code_challenge= row .try_get("code_challenge")
              .map_err(errors::any)?;
         output.   code_challenge_method= row
                .try_get("code_challenge_method")
                .map_err(errors::any)?;
         output.   nonce= row.try_get("nonce").map_err(errors::any)?;
         output.   state= row.try_get("state").map_err(errors::any)?;
         output.   hmac_key= row.try_get("hmac_key").map_err(errors::any)?;
         output.   force_approval_prompt= row .try_get("force_approval_prompt") .map_err(errors::any)?;
         output.   logged_in= row.try_get("logged_in").map_err(errors::any)?;
         output.   claim=serde_json::from_str(
            &row.try_get::<String, _>("claim").map_err(errors::any)?,
        )
        .map_err(errors::any)?;
         output.   connector_id= row.try_get("connector_id").map_err(errors::any)?;
         output.   connector_data=row
            .try_get::<Option<String>, _>("connector_data")
            .map_err(errors::any)?
            .map(|v| RawValue::from_string(v).unwrap());
         output.   expiry= row.try_get("expiry").map_err(errors::any)?;
         output.   created_at= row.try_get("created_at").map_err(errors::any)?;
         output.   updated_at= row.try_get("updated_at").map_err(errors::any)?;
 Ok(())
    }
    async fn list(
        &self,
        _opts: &Self::L,
        _output: &mut List<Self::T>,
    ) -> Result<()> {
        todo!()
    }
    fn watch<W: Watcher<Event<Self::T>>>(
        &self,
        _handler: W,
        _remove: impl Fn() + Send + 'static,
    ) -> Box<dyn Fn() + Send> {
        todo!()
    }
    async fn count(&self, _opts: &Self::L, _unscoped: bool) -> Result<i64> {
        todo!()
    }
}
