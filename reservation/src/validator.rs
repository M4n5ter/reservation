use crate::ReservationError;

pub trait Validator {
    fn validate(&self) -> Result<(), ReservationError>;
}
