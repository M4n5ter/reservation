use crate::{pb::ReservationQuery, ReservationStatus};

impl ReservationQuery {
    // / get status that can be used in sqlx query.
    pub fn get_status(&self) -> Option<String> {
        if self.status == ReservationStatus::Unknown as i32 {
            None
        } else {
            Some(
                ReservationStatus::from_i32(self.status)
                    .unwrap()
                    .to_string(),
            )
        }
    }
}
