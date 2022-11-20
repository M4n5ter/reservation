use std::ops::Bound;

use chrono::{DateTime, Utc};
use sqlx::{
    postgres::{types::PgRange, PgRow},
    types::Uuid,
    FromRow, Row,
};

use crate::{to_timestamp, Reservation, ReservationStatus, RsvpStatus};

impl FromRow<'_, PgRow> for Reservation {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let id = row.get::<Uuid, &str>("id").to_string();
        let range: PgRange<DateTime<Utc>> = row.get("timespan");
        let range: NaiveRange<DateTime<Utc>> = range.into();
        // in real world, reservation will always have a bound
        assert!(range.start.is_some());
        assert!(range.end.is_some());

        let start = range.start.unwrap();
        let end = range.end.unwrap();

        let status: RsvpStatus = row.get("status");
        Ok(Reservation {
            id,
            user_id: row.get("user_id"),
            status: ReservationStatus::from(status) as i32,
            resource_id: row.get("resource_id"),
            start: Some(to_timestamp(start)),
            end: Some(to_timestamp(end)),
            note: row.get("note"),
        })
    }
}

struct NaiveRange<T> {
    start: Option<T>,
    end: Option<T>,
}

impl<T> From<PgRange<T>> for NaiveRange<T> {
    fn from(range: PgRange<T>) -> Self {
        let f = |b: Bound<T>| match b {
            Bound::Included(v) => Some(v),
            Bound::Excluded(v) => Some(v),
            Bound::Unbounded => None,
        };
        let start = f(range.start);
        let end = f(range.end);

        Self { start, end }
    }
}
