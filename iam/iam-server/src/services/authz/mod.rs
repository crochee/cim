mod abac;

use validator::Validate;

use cim_core::Result;

#[async_trait::async_trait]
pub trait Matchers {
    /// input attributes given by Matchers
    type Attributes: Validate;
    /// authorize return  ok or error
    async fn authorize(&self, input: &Self::Attributes) -> Result<()>;
}
