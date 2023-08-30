mod errors;
mod id;
pub mod se;

pub type Result<T, E = errors::WithBacktrace> = core::result::Result<T, E>;
pub use errors::{Code, WithBacktrace};
pub use id::next_id;
