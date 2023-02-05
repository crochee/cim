use async_trait::async_trait;

use cim_core::{Error, Result};

use crate::{
    pkg::security::verify,
    repo::{users::UserSubject, DynRepository},
};

use super::{Identity, Info, PasswordConnector, RefreshConnector, Scopes};

pub struct UserIDPassword {
    repository: DynRepository,
}

impl UserIDPassword {
    pub fn new(repository: DynRepository) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl PasswordConnector for UserIDPassword {
    fn prompt(&self) -> String {
        "UserID".to_owned()
    }
    async fn login(
        &self,
        _s: &Scopes,
        info: &Info,
    ) -> Result<(Identity, bool)> {
        let p = self
            .repository
            .user()
            .get_password(&UserSubject::UserID(info.subject.clone()))
            .await?;
        if let Err(err) = verify(&p.hash, &info.password, &p.secret) {
            tracing::error!("{}", err);
            return Ok((Default::default(), false));
        };
        let email_verified = !p.email.is_empty();
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
impl RefreshConnector for UserIDPassword {
    async fn refresh(
        &self,
        _s: &Scopes,
        identity: &Identity,
    ) -> Result<Identity> {
        let p = self
            .repository
            .user()
            .get_password(&UserSubject::UserID(identity.user_id.clone()))
            .await?;
        if p.email != identity.email {
            return Err(Error::NotFound("user not found".to_owned()));
        }
        if p.mobile != identity.mobile {
            return Err(Error::NotFound("user not found".to_owned()));
        }
        let email_verified = !p.email.is_empty();
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
