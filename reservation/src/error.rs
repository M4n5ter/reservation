use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReservationError {
    #[error("DB error: {0}")]
    DBError(#[from] sqlx::Error),
    #[error("invalid reservation id")]
    InvalidReservationId,
    #[error("invalid timespan")]
    InvalidTimespan,
    #[error("invalid user id")]
    InvalidUserId,
    #[error("invalid status")]
    InvalidStatus,
    #[error("invalid resource id")]
    InvalidResourceId,
    #[error("unknown error")]
    Unknown,
}
