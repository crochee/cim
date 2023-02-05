use async_trait::async_trait;

use sqlx::MySqlPool;

use cim_core::{next_id, Error, Result};

use crate::models::{provider::Provider, ID};

#[derive(Clone)]
pub struct MariadbProviders {
    pool: MySqlPool,
}

impl MariadbProviders {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl super::ProvidersRep for MariadbProviders {
    async fn create(&self, content: &super::Content) -> Result<ID> {
        let uid = next_id().map_err(Error::any)?;
        sqlx::query!(
            r#"INSERT INTO `provider`
            (`id`,`secret`,`redirect_url`,`name`,`prompt`,`logo_url`)
            VALUES(?,?,?,?,?,?);"#,
            uid,
            content.secret,
            content.redirect_url,
            content.name,
            content.prompt,
            content.logo_url,
        )
        .execute(&self.pool)
        .await
        .map_err(Error::any)?;

        Ok(ID {
            id: uid.to_string(),
        })
    }
    async fn get(&self, id: &str) -> Result<Provider> {
        match sqlx::query!(r#"SELECT `id`,`redirect_url`,`name`,`prompt`,`logo_url` FROM `provider` WHERE `id` = ? AND `deleted` = 0"#,id)
            .map(|row| Provider {
                id: row.id.to_string(),
                redirect_url: row.redirect_url,
                name: row.name,
                prompt:row.prompt,
                logo_url: row.logo_url,
            })
            .fetch_optional(&self.pool)
            .await
            {
                Ok(v) => match v {
                    Some(value) => Ok(value),
                    None => Err(Error::NotFound("no rows".to_owned())),
                },
                Err(err) => Err(Error::any(err)),
            }
    }
    async fn list(&self) -> Result<Vec<Provider>> {
        sqlx::query!(r#"SELECT `id`,`redirect_url`,`name`,`prompt`,`logo_url` FROM `provider` WHERE `deleted` = 0"#)
            .map(|row| Provider {
                id: row.id.to_string(),
                redirect_url: row.redirect_url,
                name: row.name,
                prompt:row.prompt,
                logo_url: row.logo_url,
            })
            .fetch_all(&self.pool)
            .await
            .map_err(Error::any)
    }
}
