mod pb;
mod types;

use chrono::{DateTime, NaiveDateTime, Utc};
pub use pb::*;
use prost_types::Timestamp;

/// convert prost_types::Timestamp to utc time
pub fn to_utc_time(timestamp: &Timestamp) -> DateTime<Utc> {
    DateTime::from_utc(
        NaiveDateTime::from_timestamp_opt(timestamp.seconds, timestamp.nanos as u32).unwrap(),
        Utc,
    )
}

/// convert DateTime<FixedOffset> to prost_types::Timestamp
pub fn to_timestamp(dt: DateTime<Utc>) -> Timestamp {
    Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    }
}
