use abi::Reservation;
use async_trait::async_trait;
use chrono::{DateTime, TimeZone};
pub use error::ReservationError;
use sqlx::{postgres::types::PgRange, PgPool};

mod error;
mod manager;

type ReservationId = String;

pub struct ReservationManager {
    pool: PgPool,
}
impl ReservationManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

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

trait Validator {
    fn validate(&self) -> Result<(), ReservationError>;
}
#[async_trait]
pub trait Rsvp {
    /// 预定资源
    async fn reserve(&self, rsvp: abi::Reservation) -> Result<abi::Reservation, ReservationError>;
    /// 改变资源状态（from pending to confirm）
    async fn change_status(&self, id: ReservationId) -> Result<abi::Reservation, ReservationError>;
    /// 更新 note
    async fn update_note(
        &self,
        id: ReservationId,
        note: String,
    ) -> Result<abi::Reservation, ReservationError>;
    /// 获取资源
    async fn get(&self, id: ReservationId) -> Result<abi::Reservation, ReservationError>;
    /// 删除资源
    async fn delete(&self, id: ReservationId) -> Result<abi::Reservation, ReservationError>;
    /// 查询资源
    async fn query(
        &self,
        query: abi::ReservationQuery,
    ) -> Result<abi::Reservation, ReservationError>;
}

impl Validator for ReservationId {
    // if empty, return error
    fn validate(&self) -> Result<(), ReservationError> {
        if self.is_empty() {
            Err(ReservationError::InvalidReservationId)
        } else {
            Ok(())
        }
    }
}

impl Validator for Reservation {
    /// validate reservation's start and end time
    fn validate(&self) -> Result<(), ReservationError> {
        // if start or end is none, return error
        if self.start.is_none() || self.end.is_none() {
            return Err(ReservationError::InvalidTimespan);
        }
        // if start is after end, return error
        let start = self.start.as_ref().unwrap().seconds;
        let end = self.end.as_ref().unwrap().seconds;
        if start > end {
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
