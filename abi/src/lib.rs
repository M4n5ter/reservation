mod map;
mod pb;
use chrono::{DateTime, NaiveDateTime, Utc};
pub use pb::*;
use prost_types::Timestamp;
use std::fmt;

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

impl fmt::Display for ReservationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReservationStatus::Pending => write!(f, "pending"),
            ReservationStatus::Blocked => write!(f, "blocked"),
            ReservationStatus::Confirmed => write!(f, "confirmed"),
            ReservationStatus::Unknown => write!(f, "unknown"),
        }
    }
}

impl From<RsvpStatus> for ReservationStatus {
    fn from(status: RsvpStatus) -> Self {
        match status {
            RsvpStatus::Unknown => ReservationStatus::Unknown,
            RsvpStatus::Pending => ReservationStatus::Pending,
            RsvpStatus::Confirmed => ReservationStatus::Confirmed,
            RsvpStatus::Blocked => ReservationStatus::Blocked,
        }
    }
}

/// database equivalent of the "reservation_status" enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "reservation_status", rename_all = "lowercase")]
pub enum RsvpStatus {
    Unknown,
    Pending,
    Confirmed,
    Blocked,
}
