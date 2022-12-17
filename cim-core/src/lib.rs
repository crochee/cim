mod errors;
mod id;
pub mod se;

pub use errors::Error;
pub type Result<T, E = Error> = core::result::Result<T, E>;

pub use id::next_id;
