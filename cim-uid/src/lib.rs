pub trait Uid {
    fn uid(&self) -> String;
}

#[cfg(feature = "derive")]
pub use cim_macros::Uid;
