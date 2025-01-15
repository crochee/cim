use async_trait::async_trait;
use sqlx::{types::Json, MySqlPool, Row};

use cim_slo::{errors, Result};

use crate::{client::Client, Interface, List};

#[derive(Clone, Debug)]
pub struct ClientImpl {
    pool: MySqlPool,
}

impl ClientImpl {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Interface for ClientImpl {
    type T = Client;
    type L = ();

    #[tracing::instrument]
    async fn put(&self, input: &Self::T) -> Result<()> {
        sqlx::query(
            r#"REPLACE INTO `client`
            (`id`,`secret`,`redirect_uris`,`trusted_peers`,`name`,`logo_url`,`account_id`)
            VALUES(?,?,?,?,?,?,?);"#,
        )
        .bind(&input.id)
        .bind(&input.secret)
        .bind(Json(&input.redirect_uris))
        .bind(Json(&input.trusted_peers))
        .bind(&input.name)
        .bind(&input.logo_url)
        .bind(&input.account_id)
        .execute(&self.pool)
        .await
        .map_err(errors::any)?;
        Ok(())
    }

    #[tracing::instrument]
    async fn delete(&self, input: &Self::T) -> Result<()> {
        let id = input
            .id
            .parse::<u64>()
            .map_err(|err| errors::bad_request(&err))?;

        sqlx::query(
            r#"UPDATE `client` SET `deleted` = `id`,`deleted_at`= now()
            WHERE id = ? AND `deleted` = 0;"#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(errors::any)?;
        Ok(())
    }

    #[tracing::instrument]
    async fn get(&self, output: &mut Self::T) -> Result<()> {
        let id = output
            .id
            .parse::<u64>()
            .map_err(|err| errors::bad_request(&err))?;
        let row = match sqlx::query(
            r#"SELECT `id`,`secret`,`redirect_uris`,`trusted_peers`,`name`,`logo_url`,`account_id`,`created_at`,`updated_at`
                FROM `client`
                WHERE id = ? AND `deleted` = 0;"#,
        )
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

        output.id = row
            .try_get::<u64, _>("id")
            .map_err(errors::any)?
            .to_string();
        output.secret = row.try_get("secret").map_err(errors::any)?;
        output.redirect_uris = row
            .try_get::<Json<Vec<String>>, _>("redirect_uris")
            .map_err(errors::any)?
            .0;

        output.trusted_peers = row
            .try_get::<Json<Vec<String>>, _>("trusted_peers")
            .map_err(errors::any)?
            .0;
        output.name = row.try_get("name").map_err(errors::any)?;
        output.logo_url = row.try_get("logo_url").map_err(errors::any)?;
        output.account_id = row.try_get("account_id").map_err(errors::any)?;
        output.created_at = row.try_get("created_at").map_err(errors::any)?;
        output.updated_at = row.try_get("updated_at").map_err(errors::any)?;
        Ok(())
    }

    #[tracing::instrument]
    async fn list(
        &self,
        _opts: &Self::L,
        output: &mut List<Self::T>,
    ) -> Result<()> {
        let rows = sqlx::query(
                    r#"SELECT `id`,`secret`,`redirect_uris`,`trusted_peers`,`name`,`logo_url`,`account_id`,`created_at`,`updated_at`
                FROM `client`
                WHERE `deleted` = 0;"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(errors::any)?;
        for row in rows.iter() {
            output.data.push(Self::T {
                id: row
                    .try_get::<u64, _>("id")
                    .map_err(errors::any)?
                    .to_string(),
                account_id: row.try_get("account_id").map_err(errors::any)?,
                secret: row.try_get("secret").map_err(errors::any)?,
                logo_url: row.try_get("logo_url").map_err(errors::any)?,
                redirect_uris: row
                    .try_get::<Json<Vec<String>>, _>("redirect_uris")
                    .map_err(errors::any)?
                    .0,
                trusted_peers: row
                    .try_get::<Json<Vec<String>>, _>("trusted_peers")
                    .map_err(errors::any)?
                    .0,
                name: row.try_get("name").map_err(errors::any)?,
                created_at: row.try_get("created_at").map_err(errors::any)?,
                updated_at: row.try_get("updated_at").map_err(errors::any)?,
            });
        }

        Ok(())
    }

    async fn count(&self, _opts: &Self::L, _unscoped: bool) -> Result<i64> {
        todo!()
    }
}
