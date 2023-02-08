use std::collections::HashMap;

use async_trait::async_trait;
use http::Request;

use cim_core::{Error, Result};

use crate::{
    models::provider::Provider,
    pkg::security::verify,
    repo::{users::UserSubject, DynRepository},
    DynService,
};

use super::Token;

pub struct PasswordGrant {
    repository: DynRepository,
}

impl PasswordGrant {
    pub fn new(repository: DynRepository) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl Token for PasswordGrant {
    async fn handle<F>(
        &self,
        body: &HashMap<String, String>,
        f: F,
    ) -> Result<()>
    where
        F: FnOnce() -> (String, String) + Send,
    {
        let (client_id, client_secret) = f();
        let provider = self.repository.provider().get(&client_id).await?;
        if provider.secret == client_secret {
            return Err(Error::Unauthorized);
        }
        let nonce = body.get("nonce").unwrap_or(&"".to_owned());
        let mut has_openID_scope = false;
        for scope in body
            .get("scope")
            .unwrap_or(&"".to_owned())
            .split_ascii_whitespace()
        {
            if scope.eq("openid") {
                has_openID_scope = true;
            }
        }
        if !has_openID_scope {
            return Err(Error::Unauthorized);
        }

        let username = body.get("username").unwrap_or(&"".to_owned());
        let password = body.get("password").unwrap_or(&"".to_owned());

        let p = self
            .repository
            .user()
            .get_password(&UserSubject::UserID(username.to_owned()))
            .await?;
        if let Err(err) = verify(&p.hash, &password, &p.secret) {
            tracing::error!("{}", err);
            return Err(Error::Forbidden(err.to_string()));
        };
        let email_verified = !p.email.is_empty();
        // Ok((
        //     Identity {
        //         user_id: p.user_id,
        //         username: p.user_name,
        //         preferred_username: p.nick_name,
        //         email: p.email,
        //         mobile: p.mobile,
        //         email_verified,
        //         ..Default::default()
        //     },
        //     true,
        // ))
        Ok(())
    }
}
