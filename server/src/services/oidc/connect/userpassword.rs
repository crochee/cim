use async_trait::async_trait;

use slo::{crypto::password::verify, errors, Result};
use storage::{users::UserStore, Claim};

use super::{Identity, Info, PasswordConnector, Scopes};

pub struct UserPassword<S> {
    store: S,
}

impl<S> UserPassword<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

#[async_trait]
impl<S> PasswordConnector for UserPassword<S>
where
    S: UserStore + Send + Sync,
{
    fn prompt(&self) -> &'static str {
        "User"
    }
    fn refresh_enabled(&self) -> bool {
        false
    }
    async fn login(&self, _s: &Scopes, info: &Info) -> Result<Identity> {
        let user = self.store.get_user_password(&info.subject).await?;
        if !verify(&user.password, &info.password, &user.secret)? {
            return Err(errors::not_found("user"));
        };
        Ok(Identity {
            claim: Claim {
                sub: user.id,
                opts: user.claim,
            },
            ..Default::default()
        })
    }
    async fn refresh(
        &self,
        _s: &Scopes,
        identity: &Identity,
    ) -> Result<Identity> {
        let user = self.store.get_user_password(&identity.claim.sub).await?;
        if user.id != identity.claim.sub {
            return Err(errors::not_found("user"));
        }
        Ok(Identity {
            claim: Claim {
                sub: user.id,
                opts: user.claim,
            },
            ..Default::default()
        })
    }
}
