use chrono::{TimeZone, Utc};
use sonyflake::{Error, Sonyflake};

lazy_static::lazy_static! {
    static ref SF: Sonyflake = Sonyflake::builder().start_time(Utc.with_ymd_and_hms(2020, 1, 1,0, 0, 0).unwrap()).finalize().unwrap();
}

pub fn next_id() -> Result<u64, Error> {
    SF.clone().next_id()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_id_test() {
        assert!(next_id().is_ok());
    }
}
