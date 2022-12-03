use validator::Validate;

use cim_core::{Error, Result};

use crate::services::req::Request;

pub struct Abac;

#[async_trait::async_trait]
impl super::Matchers for Abac {
    /// input attributes given by Matchers
    type Attributes = Request;
    /// authorize return  ok or error
    async fn authorize(&self, input: &Self::Attributes) -> Result<()> {
        input.validate().map_err(Error::Validates)?;
        Ok(())
    }
}
