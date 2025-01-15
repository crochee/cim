use async_trait::async_trait;
use axum::extract::Request;
use axum_extra::headers::authorization::{Basic, Credentials};
use cim_slo::{crypto::password::verify, errors, Result};
use cim_storage::{user::User, Claim, WatchInterface};
use http::header;

use super::{CallbackConnector, Identity, Scopes};

pub struct UserPassword<S> {
    store: S,
}

impl<S> UserPassword<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

#[async_trait]
impl<S> CallbackConnector for UserPassword<S>
where
    S: WatchInterface<T = User> + Send + Sync,
{
    async fn login_url(
        &self,
        _s: &Scopes,
        callback_url: &str,
        state: &str,
    ) -> Result<String> {
        Ok(format!("/login?callback={}&state={}", callback_url, state))
    }

    /// Handle the callback to the server and return an identity.
    async fn handle_callback(
        &self,
        _s: &Scopes,
        req: Request,
    ) -> Result<Identity> {
        let hv = req
            .headers()
            .get(header::AUTHORIZATION)
            .ok_or(errors::unauthorized())?;
        let info = Basic::decode(hv).ok_or(errors::unauthorized())?;

        let mut user = User {
            id: info.username().to_string(),
            ..Default::default()
        };
        self.store.get(&mut user).await?;
        if !verify(
            &user.password.unwrap_or_default(),
            info.password(),
            &user.secret.unwrap_or_default(),
        )? {
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
    fn support_refresh(&self) -> bool {
        true
    }
    async fn refresh(
        &self,
        _s: &Scopes,
        identity: &Identity,
    ) -> Result<Identity> {
        let mut user = User {
            id: identity.claim.sub.clone(),
            ..Default::default()
        };
        self.store.get(&mut user).await?;

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
