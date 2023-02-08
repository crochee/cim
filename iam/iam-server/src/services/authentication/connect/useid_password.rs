use async_trait::async_trait;

use cim_core::{Code, Result};

use crate::{
    pkg::security::verify,
    store::{users::UserSubject, Store},
};

use super::{Identity, Info, PasswordConnector, RefreshConnector, Scopes};

pub struct UserIDPassword<S> {
    store: S,
}

impl<S> UserIDPassword<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

#[async_trait]
impl<S> PasswordConnector for UserIDPassword<S>
where
    S: Store,
{
    fn prompt(&self) -> &'static str {
        "UserID"
    }
    async fn login(
        &self,
        _s: &Scopes,
        info: &Info,
    ) -> Result<(Identity, bool)> {
        let p = self
            .store
            .user_get_password(&UserSubject::UserID(info.subject.clone()))
            .await?;
        if let Err(err) = verify(&p.hash, &info.password, &p.secret) {
            tracing::error!("{}", err);
            return Ok((Default::default(), false));
        };
        let email_verified = p.email.is_some();
        Ok((
            Identity {
                user_id: p.user_id,
                username: p.user_name,
                preferred_username: p.nick_name,
                email: p.email,
                mobile: p.mobile,
                email_verified,
                ..Default::default()
            },
            true,
        ))
    }
}

#[async_trait]
impl<S> RefreshConnector for UserIDPassword<S>
where
    S: Store + Send + Sync,
{
    async fn refresh(
        &self,
        _s: &Scopes,
        identity: &Identity,
    ) -> Result<Identity> {
        let p = self
            .store
            .user_get_password(&UserSubject::UserID(identity.user_id.clone()))
            .await?;
        if p.email != identity.email {
            return Err(Code::not_found("user"));
        }
        if p.mobile != identity.mobile {
            return Err(Code::not_found("user"));
        }
        let email_verified = p.email.is_some();
        Ok(Identity {
            user_id: p.user_id,
            username: p.user_name,
            preferred_username: p.nick_name,
            email: p.email,
            mobile: p.mobile,
            email_verified,
            ..Default::default()
        })
    }
}
