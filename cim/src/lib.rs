pub trait Uid {
    fn uid(&self) -> String;
}

#[cfg(feature = "derive")]
pub use cim_macros::Uid;

// #[cfg(feature = "derive")]
// pub use cim_macros::Opt;

pub mod pim {
    pub use cim_pim::*;
}

pub mod watch {
    pub use cim_watch::*;
}

pub mod storage {
    pub use cim_storage::*;
}

pub use cim_slo::*;
