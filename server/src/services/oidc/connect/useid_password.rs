use async_trait::async_trait;

use crate::{errors, Result};

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
            return Err(errors::not_found("user"));
        }
        if p.mobile != identity.mobile {
            return Err(errors::not_found("user"));
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

#[cfg(test)]
mod tests {
    use crate::{
        pkg::security::encrypt,
        services::authentication::connect::parse_scopes,
        store::{users::Password, MockStore},
    };

    use super::*;

    #[tokio::test]
    async fn login_test() {
        let password = String::from("sgjfasfas");
        let secret = String::from("testsss");
        let password_str = encrypt(&password, &secret).unwrap();
        let mut store = MockStore::new();
        store.expect_user_get_password().returning(move |_v| {
            Ok(Password {
                user_id: "123".to_string(),
                user_name: "test".to_string(),
                nick_name: "tss".to_string(),
                email: None,
                mobile: None,
                hash: password_str.clone(),
                secret: secret.clone(),
            })
        });
        let up = UserIDPassword::new(store);

        let scopes = vec!["openid".to_string()];
        let (identity, ok) = up
            .login(
                &parse_scopes(&scopes),
                &Info {
                    subject: "123".to_string(),
                    password,
                },
            )
            .await
            .unwrap();
        assert!(ok);
        println!("{:#?}", identity);
    }
}
