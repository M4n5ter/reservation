use abi::to_utc_time;
use async_trait::async_trait;
use sqlx::{postgres::types::PgRange, types::Uuid, Row};

use crate::{ReservationError, ReservationId, ReservationManager, Rsvp, Validator, Window};

#[async_trait]
impl Rsvp for ReservationManager {
    /// 预定资源，插入一条预定记录
    async fn reserve(
        &self,
        mut rsvp: abi::Reservation,
    ) -> Result<abi::Reservation, ReservationError> {
        rsvp.validate()?;

        let status = abi::ReservationStatus::from_i32(rsvp.status)
            .unwrap_or(abi::ReservationStatus::Pending);
        let start = to_utc_time(rsvp.start.as_ref().unwrap());
        let end = to_utc_time(rsvp.end.as_ref().unwrap());
        let window = Window::new(start, end);
        let timespan = PgRange::from(window);

        let id: Uuid = sqlx::query(
            "INSERT INTO rsvp.reservations (user_id, status, resource_id, timespan, note) VALUES ($1, $2::rsvp.reservation_status, $3, $4, $5) RETURNING id",
        )
        .bind(rsvp.user_id.clone())
        .bind(status.to_string())
        .bind(rsvp.resource_id.clone())
        .bind(timespan)
        .bind(rsvp.note.clone())
        .fetch_one(&self.pool)
        .await?
        .get(0);
        rsvp.id = id.to_string();

        Ok(rsvp)
    }

    async fn change_status(
        &self,
        _id: ReservationId,
    ) -> Result<abi::Reservation, ReservationError> {
        todo!()
    }
    async fn update_note(
        &self,
        _id: ReservationId,
        _note: String,
    ) -> Result<abi::Reservation, ReservationError> {
        todo!()
    }
    async fn get(&self, _id: ReservationId) -> Result<abi::Reservation, ReservationError> {
        todo!()
    }
    async fn delete(&self, _id: ReservationId) -> Result<abi::Reservation, ReservationError> {
        todo!()
    }
    async fn query(
        &self,
        _query: abi::ReservationQuery,
    ) -> Result<abi::Reservation, ReservationError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use abi::to_timestamp;
    use chrono::{DateTime, FixedOffset, Utc};

    use super::*;
    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_should_work_for_valid_window() {
        let manager = ReservationManager::new(migrated_pool.clone());
        let start: DateTime<FixedOffset> = "2022-11-18T12:00:00+0800".parse().unwrap();
        let end: DateTime<FixedOffset> = "2022-11-20T14:00:00+0800".parse().unwrap();

        let rsvp = abi::Reservation {
            id: "".to_string(),
            user_id: "M4n5ter".to_string(),
            status: abi::ReservationStatus::Pending as i32,
            resource_id: "hotel room 1 ".to_string(),
            start: Some(to_timestamp(start.with_timezone(&Utc))),
            end: Some(to_timestamp(end.with_timezone(&Utc))),
            note: "I'll arrive at 3PM.Please hold".to_string(),
        };
        let rsvp = manager.reserve(rsvp).await.unwrap();
        assert!(!rsvp.id.is_empty());
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_should_not_work_for_conflict_window() {
        let manager = ReservationManager::new(migrated_pool.clone());
        let start: DateTime<FixedOffset> = "2022-11-18T12:00:00+0800".parse().unwrap();
        let end: DateTime<FixedOffset> = "2022-11-20T14:00:00+0800".parse().unwrap();
        let rsvp = abi::Reservation {
            id: "".to_string(),
            user_id: "M4n5ter".to_string(),
            status: abi::ReservationStatus::Pending as i32,
            resource_id: "hotel room 1 ".to_string(),
            start: Some(to_timestamp(start.with_timezone(&Utc))),
            end: Some(to_timestamp(end.with_timezone(&Utc))),
            note: "I'll arrive at 3PM.Please hold".to_string(),
        };
        manager.reserve(rsvp).await.unwrap();

        let start: DateTime<FixedOffset> = "2022-11-17T12:00:00+0800".parse().unwrap();
        let end: DateTime<FixedOffset> = "2022-11-29T14:00:00+0800".parse().unwrap();
        let rsvp = abi::Reservation {
            id: "".to_string(),
            user_id: "Syuu".to_string(),
            status: abi::ReservationStatus::Pending as i32,
            resource_id: "hotel room 1 ".to_string(),
            start: Some(to_timestamp(start.with_timezone(&Utc))),
            end: Some(to_timestamp(end.with_timezone(&Utc))),
            note: "I'll arrive at 3PM.Please hold".to_string(),
        };
        let err = manager.reserve(rsvp).await.unwrap_err();
        println!("{:?}", err);
    }
}
