use async_trait::async_trait;

use cim_core::Result;

use crate::{
    models::{claim::Claims, provider::Provider, ID},
    repo::{authreqs, providers::Content, DynRepository},
};

use super::{
    Authenticator, Identity, Info, PasswordConnector, Scopes, TokenOpts,
};

pub struct IAMAuthenticator {
    repository: DynRepository,
}

impl IAMAuthenticator {
    pub fn new(repository: DynRepository) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl Authenticator for IAMAuthenticator {
    fn authreq(&self) -> authreqs::DynAuthReqs {
        self.repository.authreq()
    }
    async fn create_provider(&self, content: &Content) -> Result<ID> {
        self.repository.provider().create(content).await
    }
    async fn providers(&self) -> Result<Vec<Provider>> {
        self.repository.provider().list().await
    }
    async fn get_provider(&self, id: &str) -> Result<Provider> {
        self.repository.provider().get(id).await
    }
    async fn login(&self, s: &Scopes, info: &Info) -> Result<(Identity, bool)> {
        super::password::UserIDPassword::new(self.repository.clone())
            .login(s, info)
            .await
    }
    async fn token(
        &self,
        claims: &Claims,
        opts: &TokenOpts,
    ) -> Result<(String, i64)> {
        todo!()
    }
    async fn verify(&self, token: &str) -> Result<Claims> {
        todo!()
    }
}
