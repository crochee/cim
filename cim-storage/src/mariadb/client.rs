use async_trait::async_trait;
use sqlx::{MySqlPool, Row};

use cim_slo::{errors, Result};
use cim_watch::Watcher;

use crate::{client::Client, Event, Interface, List};

#[derive(Clone)]
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
    async fn put(&self, input: &Self::T, _ttl: u64) -> Result<()> {
        let redirect_uris =
            serde_json::to_string(&input.redirect_uris).map_err(errors::any)?;

        let trusted_peers =
            serde_json::to_string(&input.trusted_peers).map_err(errors::any)?;

        let account_id = input
            .account_id
            .parse::<u64>()
            .map_err(|err| errors::bad_request(&err))?;

        sqlx::query(
            r#"REPLACE INTO `client`
            (`id`,`secret`,`redirect_uris`,`trusted_peers`,`name`,`logo_url`,`account_id`)
            VALUES(?,?,?,?,?,?,?);"#,
        )
        .bind(&input.id)
        .bind(&input.secret)
        .bind(redirect_uris)
        .bind(trusted_peers)
        .bind(&input.name)
        .bind(&input.logo_url)
        .bind(account_id)
        .execute(&self.pool)
        .await
        .map_err(errors::any)?;
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<()> {
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
    async fn get(&self, id: &str, output: &mut Self::T) -> Result<()> {
        let id = id.parse::<u64>().map_err(|err| errors::bad_request(&err))?;
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
        output.redirect_uris = serde_json::from_str(
            &row.try_get::<String, _>("redirect_uris")
                .map_err(errors::any)?,
        )
        .map_err(errors::any)?;

        output.trusted_peers = serde_json::from_str(
            &row.try_get::<String, _>("trusted_peers")
                .map_err(errors::any)?,
        )
        .map_err(errors::any)?;
        output.name = row.try_get("name").map_err(errors::any)?;
        output.logo_url = row.try_get("logo_url").map_err(errors::any)?;
        output.account_id = row
            .try_get::<u64, _>("account_id")
            .map_err(errors::any)?
            .to_string();
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
