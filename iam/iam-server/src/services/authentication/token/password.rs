use async_trait::async_trait;
use http::Request;

use cim_core::Result;

use crate::models::provider::Provider;

use super::Token;

pub struct PasswordGrant {}

#[async_trait]
impl<B: Send + 'static> Token<B> for PasswordGrant {
    async fn client(req: Request<B>) -> Result<Provider> {
        todo!()
    }

    async fn handle(req: http::Request<B>) -> Result<()> {
        todo!()
    }
}
