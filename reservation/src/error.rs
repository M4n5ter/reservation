use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReservationError {
    #[error("DB error: {0}")]
    DBError(#[from] sqlx::Error),
    #[error("invalid reservation id")]
    InvalidReservationId,
    #[error("invalid timespan")]
    InvalidTimespan,
    #[error("invalid user id: {0}")]
    InvalidUserId(#[from] sqlx::types::uuid::Error),
    #[error("invalid status")]
    InvalidStatus,
    #[error("invalid resource id: {0}")]
    InvalidResourceId(String),
    #[error("unknown error")]
    Unknown,
}
