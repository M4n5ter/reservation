use abi::{to_utc_time, Reservation, ReservationQuery};
use chrono::{DateTime, TimeZone};
use sqlx::postgres::types::PgRange;

use crate::{validator::Validator, ReservationError};

pub struct Window<T>
where
    T: TimeZone,
{
    start: DateTime<T>,
    end: DateTime<T>,
}
impl<T: TimeZone> Window<T> {
    pub fn new(start: DateTime<T>, end: DateTime<T>) -> Self {
        Self { start, end }
    }
}
impl Window<chrono::Utc> {
    pub fn from_reservation(reservation: &Reservation) -> Self {
        // start and end shouldn't be None in real world.
        Self {
            start: to_utc_time(reservation.start.as_ref().unwrap()),
            end: to_utc_time(reservation.end.as_ref().unwrap()),
        }
    }

    pub fn from_query(query: &ReservationQuery) -> Self {
        Self {
            start: to_utc_time(query.start.as_ref().unwrap()),
            end: to_utc_time(query.end.as_ref().unwrap()),
        }
    }
}

impl<T: TimeZone> Validator for Window<T> {
    fn validate(&self) -> Result<(), ReservationError> {
        if self.start > self.end {
            return Err(ReservationError::InvalidTimespan);
        }
        Ok(())
    }
}

// convert Window to PgRange
impl<T> From<Window<T>> for PgRange<DateTime<T>>
where
    T: TimeZone,
{
    fn from(window: Window<T>) -> Self {
        PgRange {
            start: std::ops::Bound::Excluded(window.start),
            end: std::ops::Bound::Excluded(window.end),
        }
    }
}
