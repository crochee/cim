use async_trait::async_trait;
use sqlx::{MySqlPool, Row};

use cim_slo::{errors, next_id, Result};

use crate::ID;

use super::{Client, ClientStore};

pub struct ClientImpl {
    pool: MySqlPool,
}

impl ClientImpl {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ClientStore for ClientImpl {
    async fn put_client(&self, content: &Client) -> Result<ID> {
        let id = if content.id.is_empty() {
            next_id().map_err(errors::any)?
        } else {
            content
                .id
                .parse::<u64>()
                .map_err(|err| errors::bad_request(&err))?
        };

        let redirect_uris = serde_json::to_string(&content.redirect_uris)
            .map_err(errors::any)?;

        let trusted_peers = serde_json::to_string(&content.trusted_peers)
            .map_err(errors::any)?;

        let account_id = content
            .account_id
            .parse::<u64>()
            .map_err(|err| errors::bad_request(&err))?;

        sqlx::query(
            r#"REPLACE INTO `client`
            (`id`,`secret`,`redirect_uris`,`trusted_peers`,`name`,`logo_url`,`account_id`)
            VALUES(?,?,?,?,?,?,?);"#,
        )
        .bind(id)
        .bind(&content.secret)
        .bind(redirect_uris)
        .bind(trusted_peers)
        .bind(&content.name)
        .bind(&content.logo_url)
        .bind(account_id)
        .execute(&self.pool)
        .await
        .map_err(errors::any)?;
        Ok(ID { id: id.to_string() })
    }
    async fn get_client(
        &self,
        id: &str,
        account_id: Option<String>,
    ) -> Result<Client> {
        let mut wheres = format!(
            r#"`id` = {}"#,
            id.parse::<u64>().map_err(|err| errors::bad_request(&err))?
        );
        if let Some(v) = account_id {
            let account_id_u64: u64 =
                v.parse().map_err(|err| errors::bad_request(&err))?;
            wheres.push_str(" AND ");
            wheres.push_str(
                format!(r#"`account_id` = {}"#, account_id_u64).as_str(),
            );
        }
        let row = match sqlx::query(format!(
            r#"SELECT `id`,`secret`,`redirect_uris`,`trusted_peers`,`name`,`logo_url`,`account_id`,`created_at`,`updated_at`
            FROM `client`
            WHERE {wheres} AND deleted = 0;"#).as_str())
        .fetch_optional(&self.pool)
        .await
        {
            Ok(v) => match v {
                Some(value) => Ok(value),
                None => Err(errors::not_found("no rows")),
            },
            Err(err) => Err(errors::any(err)),
        }?;

        let redirect_uris = serde_json::from_str(
            &row.try_get::<String, _>("redirect_uris")
                .map_err(errors::any)?,
        )
        .map_err(errors::any)?;

        let trusted_peers = serde_json::from_str(
            &row.try_get::<String, _>("trusted_peers")
                .map_err(errors::any)?,
        )
        .map_err(errors::any)?;
        Ok(Client {
            id: row
                .try_get::<u64, _>("id")
                .map_err(errors::any)?
                .to_string(),
            secret: row.try_get("secret").map_err(errors::any)?,
            redirect_uris,
            trusted_peers,
            name: row.try_get("name").map_err(errors::any)?,
            logo_url: row.try_get("logo_url").map_err(errors::any)?,
            account_id: row
                .try_get::<u64, _>("account_id")
                .map_err(errors::any)?
                .to_string(),
            created_at: row.try_get("created_at").map_err(errors::any)?,
            updated_at: row.try_get("updated_at").map_err(errors::any)?,
        })
    }
    async fn delete_client(
        &self,
        id: &str,
        account_id: Option<String>,
    ) -> Result<()> {
        let mut wheres = format!(
            r#"`id` = {}"#,
            id.parse::<u64>().map_err(|err| errors::bad_request(&err))?
        );
        if let Some(v) = account_id {
            let account_id_u64: u64 =
                v.parse().map_err(|err| errors::bad_request(&err))?;
            wheres.push_str(" AND ");
            wheres.push_str(
                format!(r#"`account_id` = {}"#, account_id_u64).as_str(),
            );
        }
        sqlx::query(
            format!(
                r#"UPDATE `client` SET `deleted` = `id`,`deleted_at`= now()
            WHERE {wheres} AND `deleted` = 0;"#,
            )
            .as_str(),
        )
        .execute(&self.pool)
        .await
        .map_err(errors::any)?;
        Ok(())
    }
}
