mod error;
mod manager;
mod validator;
mod window;

use abi::Reservation;
use async_trait::async_trait;
pub use error::ReservationError;
use sqlx::{types::Uuid, PgPool};
use std::str::FromStr;
use validator::Validator;
use window::Window;

pub type ReservationId = Uuid;

pub struct ReservationManager {
    pool: PgPool,
}
impl ReservationManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
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
    ) -> Result<Vec<abi::Reservation>, ReservationError>;
}

impl Validator for ReservationId {
    // if empty, return error
    fn validate(&self) -> Result<(), ReservationError> {
        if self.is_nil() {
            Err(ReservationError::InvalidReservationId)
        } else {
            Ok(())
        }
    }
}

impl Validator for Reservation {
    /// validate a reservation
    fn validate(&self) -> Result<(), ReservationError> {
        // validate timespan
        Window::from_reservation(self).validate()?;
        // validate reservation id
        ReservationId::from_str(&self.id).unwrap().validate()?;
        Ok(())
    }
}
