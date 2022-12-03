mod errors;

pub use errors::Error;
pub type Result<T, E = Error> = core::result::Result<T, E>;
