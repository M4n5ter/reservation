use crate::window::Window;
use crate::{ReservationError, ReservationId, ReservationManager, Rsvp, Validator};
use async_trait::async_trait;
use sqlx::{postgres::types::PgRange, types::Uuid, Row};

#[async_trait]
impl Rsvp for ReservationManager {
    /// Create a new reservation.
    async fn reserve(
        &self,
        mut rsvp: abi::Reservation,
    ) -> Result<abi::Reservation, ReservationError> {
        let window = Window::from_reservation(&rsvp);
        window.validate()?;

        let status = rsvp.get_status();
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

    /// change pending status to confirmed status.
    async fn change_status(&self, id: ReservationId) -> Result<abi::Reservation, ReservationError> {
        let rsvp = sqlx::query_as(
            "UPDATE rsvp.reservations SET status = 'confirmed' WHERE id = $1::UUID and status = 'pending' RETURNING *",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(rsvp)
    }

    /// update reservation's note.
    async fn update_note(
        &self,
        id: ReservationId,
        note: String,
    ) -> Result<abi::Reservation, ReservationError> {
        let rsvp = sqlx::query_as(
            "UPDATE rsvp.reservations SET note = $1 WHERE id = $2::UUID RETURNING *",
        )
        .bind(note)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(rsvp)
    }

    /// get a reservation by id.
    async fn get(&self, id: ReservationId) -> Result<abi::Reservation, ReservationError> {
        let rsvp = sqlx::query_as("SELECT * FROM rsvp.reservations WHERE id = $1::UUID")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(rsvp)
    }

    /// delete a reservation by id.
    async fn delete(&self, id: ReservationId) -> Result<abi::Reservation, ReservationError> {
        let rsvp = sqlx::query_as("DELETE FROM rsvp.reservations WHERE id = $1::UUID RETURNING *")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(rsvp)
    }

    /// query a reservation
    async fn query(
        &self,
        query: abi::ReservationQuery,
    ) -> Result<Vec<abi::Reservation>, ReservationError> {
        // convert query.start and query.end to PgRange
        let window = Window::from_query(&query);
        let timespan = PgRange::from(window);
        let status = query.get_status();

        let user_id: Option<String> = {
            if query.user_id.is_empty() {
                None
            } else {
                Some(query.user_id)
            }
        };

        let resource_id = {
            if query.resource_id.is_empty() {
                None
            } else {
                Some(query.resource_id)
            }
        };

        let desc: Option<bool> = {
            if query.desc {
                Some(true)
            } else {
                None
            }
        };

        let page: Option<i64> = {
            if query.page > 0 {
                Some(query.page)
            } else {
                None
            }
        };
        let page_size: Option<i64> = {
            if query.page_size > 0 {
                Some(query.page_size)
            } else {
                None
            }
        };

        let rsvp: Vec<abi::Reservation> = sqlx::query_as(
            "SELECT * FROM rsvp.query ($1, $2, $3::TSTZRANGE, $4::rsvp.reservation_status, $5::INTEGER, $6, $7::INTEGER)"
        )
        .bind(user_id)
        .bind(resource_id)
        .bind(timespan)
        .bind(status)
        .bind(page)
        .bind(desc)
        .bind(page_size)
        .fetch_all(&self.pool)
        .await?;
        Ok(rsvp)
    }
}

#[cfg(test)]
mod tests {

    use std::str::FromStr;

    use super::*;
    use abi::{to_timestamp, ReservationQueryBuilder, ReservationStatus};
    use chrono::{DateTime, FixedOffset, Utc};
    use prost_types::Timestamp;

    /// generate a pending reservation
    fn generate_resource(
        user_id: &str,
        resource_id: &str,
        start: &str,
        end: &str,
        note: &str,
    ) -> abi::Reservation {
        let start_dt: DateTime<FixedOffset>;
        let end_dt: DateTime<FixedOffset>;
        if !start.is_empty() && !end.is_empty() {
            start_dt = start.parse().unwrap();
            end_dt = end.parse().unwrap();
        } else {
            start_dt = Utc::now().into();
            end_dt = start_dt + chrono::Duration::days(1);
        }

        abi::Reservation {
            id: "".to_string(),
            user_id: user_id.to_string(),
            status: abi::ReservationStatus::Pending as i32,
            resource_id: resource_id.to_string(),
            start: Some(to_timestamp(start_dt.with_timezone(&Utc))),
            end: Some(to_timestamp(end_dt.with_timezone(&Utc))),
            note: note.to_string(),
        }
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_should_work_for_valid_window() {
        let manager = ReservationManager::new(migrated_pool.clone());
        let rsvp = generate_resource(
            "M4n5ter",
            "hotel room 1",
            "2022-11-18T12:00:00+0800",
            "2022-11-20T14:00:00+0800",
            "I'll arrive at 3PM.Please hold",
        );

        let rsvp = manager.reserve(rsvp).await.unwrap();
        assert!(!rsvp.id.is_empty());
    }

    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn reserve_should_not_work_for_conflict_window() {
        let manager = ReservationManager::new(migrated_pool.clone());
        let rsvp = generate_resource(
            "M4n5ter",
            "hotel room 1",
            "2022-11-18T12:00:00+0800",
            "2022-11-20T14:00:00+0800",
            "I'll arrive at 3PM.Please hold",
        );

        manager.reserve(rsvp).await.unwrap();

        let rsvp = generate_resource(
            "Syuu",
            "hotel room 1",
            "2022-11-17T12:00:00+0800",
            "2022-11-29T14:00:00+0800",
            "I'll arrive at 3PM.Please hold",
        );

        let err = manager.reserve(rsvp).await;
        // TODO: check error type
        assert!(err.is_err());
    }

    /// change status should work for pending reservation
    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn change_status_should_work_for_pending_reservation() {
        let manager = ReservationManager::new(migrated_pool.clone());
        let rsvp = generate_resource(
            "Syuu",
            "hotel room 1",
            "2022-11-17T12:00:00+0800",
            "2022-11-29T14:00:00+0800",
            "I'll arrive at 3PM.Please hold",
        );
        let rsvp = manager.reserve(rsvp).await.unwrap();
        let rsvp = manager
            .change_status(rsvp.id.parse().unwrap())
            .await
            .unwrap();
        assert_eq!(rsvp.status, abi::ReservationStatus::Confirmed as i32);
    }

    /// query should work
    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn query_should_work() {
        let manager = ReservationManager::new(migrated_pool.clone());

        // STEP 1: reserve a reservation, change status to confirmed and query it.
        let rsvp_1 = generate_resource(
            "Syuu",
            "hotel room 1",
            "2022-11-13T12:00:00+0800",
            "2022-11-16T14:00:00+0800",
            "I'll arrive at 12AM.Please hold",
        );
        let rsvp_1 = manager.reserve(rsvp_1).await.unwrap();
        manager
            .change_status(rsvp_1.id.parse().unwrap())
            .await
            .unwrap();

        let query = ReservationQueryBuilder::default()
            .user_id("Syuu")
            .resource_id("hotel room 1")
            .start("2022-11-13T12:00:00+0800".parse::<Timestamp>().unwrap())
            .end("2022-11-16T14:00:00+0800".parse::<Timestamp>().unwrap())
            .status(ReservationStatus::Confirmed as i32)
            .build()
            .unwrap();

        let rsvps = manager.query(query).await.unwrap();
        assert_eq!(rsvps.len(), 1);
        assert_eq!(rsvps[0].id, rsvp_1.id);

        // STEP 2: reserve another one whose status is pending and query it.
        let rsvp_2 = generate_resource(
            "Syuu",
            "hotel room 1",
            "2022-12-17T15:00:00+0800",
            "2022-12-25T14:00:00+0800",
            "I'll arrive at 3PM.Please hold",
        );
        let rsvp_2 = manager.reserve(rsvp_2).await.unwrap();
        let query = ReservationQueryBuilder::default()
            .user_id("Syuu")
            .resource_id("hotel room 1")
            .start("2022-12-16T12:00:00+0800".parse::<Timestamp>().unwrap())
            .end("2022-12-26T14:00:00+0800".parse::<Timestamp>().unwrap())
            .status(ReservationStatus::Pending as i32)
            .build()
            .unwrap();
        let rsvps = manager.query(query).await.unwrap();
        assert_eq!(rsvps.len(), 1);
        assert_eq!(rsvps[0].id, rsvp_2.id);

        // STEP 3: generate three pending reservations that the user is not Syuu and resource is not hotel room 1,then query two of them by time range.
        let rsvp_3 = generate_resource(
            "M4n5ter",
            "hotel room 2",
            "2022-12-17T15:00:00+0800",
            "2022-12-25T14:00:00+0800",
            "I'll arrive at 3PM.Please hold",
        );
        let rsvp_3 = manager.reserve(rsvp_3).await.unwrap();
        let rsvp_4 = generate_resource(
            "M4n5ter",
            "hotel room 2",
            "2023-01-02T15:00:00+0800",
            "2023-01-03T14:00:00+0800",
            "I'll arrive at 3PM.Please hold",
        );
        let rsvp_4 = manager.reserve(rsvp_4).await.unwrap();
        let rsvp_5 = generate_resource(
            "M4n5ter",
            "hotel room 2",
            "2023-01-04T15:00:00+0800",
            "2023-01-05T14:00:00+0800",
            "I'll arrive at 3PM.Please hold",
        );
        let rsvp_5 = manager.reserve(rsvp_5).await.unwrap();
        // make a query and the result should be rsvp_1 and rsvp_2.
        let query = ReservationQueryBuilder::default()
            .user_id("M4n5ter")
            .resource_id("hotel room 2")
            .start("2022-12-16T12:00:00+0800".parse::<Timestamp>().unwrap())
            .end("2023-01-04T14:00:00+0800".parse::<Timestamp>().unwrap())
            .status(ReservationStatus::Pending as i32)
            .build()
            .unwrap();
        let rsvps = manager.query(query).await.unwrap();
        assert_eq!(rsvps.len(), 2);
        assert_eq!(rsvps[0].id, rsvp_3.id);
        assert_eq!(rsvps[1].id, rsvp_4.id);

        // STEP 4: generate two reservations,one's user is Syuu and resource is hotel room 2,another's user is M4n5ter and resource is hotel room 1,
        // both of them are confirmed,and query all reservations above by different conditions.
        let rsvp_6 = generate_resource(
            "Syuu",
            "hotel room 2",
            "2023-01-17T15:00:00+0800",
            "2023-01-25T14:00:00+0800",
            "I'll arrive at 3PM.Please hold",
        );
        let rsvp_6 = manager.reserve(rsvp_6).await.unwrap();
        manager
            .change_status(rsvp_6.id.parse().unwrap())
            .await
            .unwrap();
        let rsvp_7 = generate_resource(
            "M4n5ter",
            "hotel room 1",
            "2023-01-17T15:00:00+0800",
            "2023-01-25T14:00:00+0800",
            "I'll arrive at 3PM.Please hold",
        );
        let rsvp_7 = manager.reserve(rsvp_7).await.unwrap();
        manager
            .change_status(rsvp_7.id.parse().unwrap())
            .await
            .unwrap();
        let query = ReservationQueryBuilder::default()
            .resource_id("hotel room 1")
            .start("2021-01-16T12:00:00+0800".parse::<Timestamp>().unwrap())
            .end("2025-01-26T14:00:00+0800".parse::<Timestamp>().unwrap())
            .status(ReservationStatus::Pending as i32)
            .build()
            .unwrap();
        let rsvps = manager.query(query).await.unwrap();
        assert_eq!(rsvps.len(), 1);
        assert_eq!(rsvps[0].id, rsvp_2.id);

        // STEP 6: query all confirmed reservations within time range whose resource is hotel room 1, order by start time desc.
        let query = ReservationQueryBuilder::default()
            .resource_id("hotel room 1")
            .start("2021-01-16T12:00:00+0800".parse::<Timestamp>().unwrap())
            .end("2025-01-26T14:00:00+0800".parse::<Timestamp>().unwrap())
            .status(ReservationStatus::Confirmed as i32)
            .desc(true)
            .build()
            .unwrap();
        let rsvps = manager.query(query).await.unwrap();
        assert_eq!(rsvps.len(), 2);
        assert_eq!(rsvps[0].id, rsvp_7.id);
        assert_eq!(rsvps[1].id, rsvp_1.id);

        // STEP 7: query all pending reservations within time range whose resource is hotel room 2, desc.
        let query = ReservationQueryBuilder::default()
            .resource_id("hotel room 2")
            .start("2021-01-16T12:00:00+0800".parse::<Timestamp>().unwrap())
            .end("2025-01-26T14:00:00+0800".parse::<Timestamp>().unwrap())
            .status(ReservationStatus::Pending as i32)
            .desc(true)
            .build()
            .unwrap();
        let rsvps = manager.query(query).await.unwrap();
        assert_eq!(rsvps.len(), 3);
        assert_eq!(rsvps[0].id, rsvp_5.id);
        assert_eq!(rsvps[1].id, rsvp_4.id);
        assert_eq!(rsvps[2].id, rsvp_3.id);

        // STEP 8: query all confirmed reservations within time range whose resource is hotel room 2.
        let query = ReservationQueryBuilder::default()
            .resource_id("hotel room 2")
            .start("2021-01-16T12:00:00+0800".parse::<Timestamp>().unwrap())
            .end("2025-01-26T14:00:00+0800".parse::<Timestamp>().unwrap())
            .status(ReservationStatus::Confirmed as i32)
            .build()
            .unwrap();
        let rsvps = manager.query(query).await.unwrap();
        assert_eq!(rsvps.len(), 1);
        assert_eq!(rsvps[0].id, rsvp_6.id);

        // STEP 9: query all pending reservations within time range whose user is Syuu.
        let query = ReservationQueryBuilder::default()
            .user_id("Syuu")
            .start("2021-01-16T12:00:00+0800".parse::<Timestamp>().unwrap())
            .end("2025-01-26T14:00:00+0800".parse::<Timestamp>().unwrap())
            .status(ReservationStatus::Pending as i32)
            .build()
            .unwrap();
        let rsvps = manager.query(query).await.unwrap();
        assert_eq!(rsvps.len(), 1);
        assert_eq!(rsvps[0].id, rsvp_2.id);

        // STEP 10: query all confirmed reservations within time range whose user is Syuu.
        let query = ReservationQueryBuilder::default()
            .user_id("Syuu")
            .start("2021-01-16T12:00:00+0800".parse::<Timestamp>().unwrap())
            .end("2025-01-26T14:00:00+0800".parse::<Timestamp>().unwrap())
            .status(ReservationStatus::Confirmed as i32)
            .build()
            .unwrap();
        let rsvps = manager.query(query).await.unwrap();
        assert_eq!(rsvps.len(), 2);
        assert_eq!(rsvps[0].id, rsvp_1.id);
        assert_eq!(rsvps[1].id, rsvp_6.id);

        // STEP 11: query all pending reservations within time range whose user is M4n5ter.
        let query = ReservationQueryBuilder::default()
            .user_id("M4n5ter")
            .start("2021-01-16T12:00:00+0800".parse::<Timestamp>().unwrap())
            .end("2025-01-26T14:00:00+0800".parse::<Timestamp>().unwrap())
            .status(ReservationStatus::Pending as i32)
            .build()
            .unwrap();
        let rsvps = manager.query(query).await.unwrap();
        assert_eq!(rsvps.len(), 3);
        assert_eq!(rsvps[0].id, rsvp_3.id);
        assert_eq!(rsvps[1].id, rsvp_4.id);
        assert_eq!(rsvps[2].id, rsvp_5.id);

        // STEP 12: query all confirmed reservations within time range whose user is M4n5ter.
        let query = ReservationQueryBuilder::default()
            .user_id("M4n5ter")
            .start("2021-01-16T12:00:00+0800".parse::<Timestamp>().unwrap())
            .end("2025-01-26T14:00:00+0800".parse::<Timestamp>().unwrap())
            .status(ReservationStatus::Confirmed as i32)
            .build()
            .unwrap();
        let rsvps = manager.query(query).await.unwrap();
        assert_eq!(rsvps.len(), 1);
        assert_eq!(rsvps[0].id, rsvp_7.id);

        // STEP 13: query all pending reservations within time range.
        let query = ReservationQueryBuilder::default()
            .status(ReservationStatus::Pending as i32)
            .start("2021-01-16T12:00:00+0800".parse::<Timestamp>().unwrap())
            .end("2025-01-26T14:00:00+0800".parse::<Timestamp>().unwrap())
            .build()
            .unwrap();
        let rsvps = manager.query(query).await.unwrap();
        assert_eq!(rsvps.len(), 4);
        assert_eq!(rsvps[0].id, rsvp_2.id);
        assert_eq!(rsvps[1].id, rsvp_3.id);
        assert_eq!(rsvps[2].id, rsvp_4.id);
        assert_eq!(rsvps[3].id, rsvp_5.id);

        // STEP 14: query all confirmed reservations within time range.
        let query = ReservationQueryBuilder::default()
            .status(ReservationStatus::Confirmed as i32)
            .start("2021-01-16T12:00:00+0800".parse::<Timestamp>().unwrap())
            .end("2025-01-26T14:00:00+0800".parse::<Timestamp>().unwrap())
            .build()
            .unwrap();
        let rsvp = manager.query(query).await.unwrap();
        assert_eq!(rsvp.len(), 3);
        assert_eq!(rsvp[0].id, rsvp_1.id);
        assert_eq!(rsvp[1].id, rsvp_6.id);
        assert_eq!(rsvp[2].id, rsvp_7.id);
    }

    /// update_note should work
    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn update_note_should_work() {
        let manager = ReservationManager::new(migrated_pool.clone());
        // generate a pending resource
        let rsvp_8 = generate_resource(
            "Syuu",
            "hotel room 2",
            "2021-01-16T12:00:00+0800",
            "2025-01-26T14:00:00+0800",
            "note 1",
        );

        let rsvp_8 = manager.reserve(rsvp_8).await.unwrap();
        let rsvp_8 = manager
            .update_note(
                Uuid::from_str(&rsvp_8.id).unwrap(),
                "note 2 AND DROP TABLE rsvp.reservations CASCADE -- + ???".into(),
            )
            .await
            .unwrap();
        assert_eq!(
            rsvp_8.note,
            "note 2 AND DROP TABLE rsvp.reservations CASCADE -- + ???"
        );
    }

    /// get should work
    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn get_should_work() {
        let manager = ReservationManager::new(migrated_pool.clone());
        // generate a pending resource
        let rsvp_9 = generate_resource(
            "M4n5ter",
            "class room 1",
            "1991-01-16T12:00:00+0800",
            "1992-01-26T14:00:00+0800",
            "note 1",
        );

        let rsvp_9 = manager.reserve(rsvp_9).await.unwrap();
        let rsvp_9 = manager
            .change_status(Uuid::from_str(&rsvp_9.id).unwrap())
            .await
            .unwrap();
        let rsvp_9 = manager
            .get(Uuid::from_str(&rsvp_9.id).unwrap())
            .await
            .unwrap();
        assert_eq!(rsvp_9.user_id, "M4n5ter");
        assert_eq!(rsvp_9.resource_id, "class room 1");
        assert_eq!(
            rsvp_9.start,
            Some("1991-01-16T12:00:00+0800".parse::<Timestamp>().unwrap())
        );
        assert_eq!(
            rsvp_9.end,
            Some("1992-01-26T14:00:00+0800".parse::<Timestamp>().unwrap())
        );
        assert_eq!(rsvp_9.note, "note 1");
        assert_eq!(rsvp_9.status, ReservationStatus::Confirmed as i32);
    }

    /// delete should work
    #[sqlx_database_tester::test(pool(variable = "migrated_pool", migrations = "../migrations"))]
    async fn delete_should_work() {
        let manager = ReservationManager::new(migrated_pool.clone());
        // generate a pending resource
        let rsvp_10 = generate_resource(
            "M4n5ter",
            "class room 1",
            "1991-01-16T12:00:00+0800",
            "1992-01-26T14:00:00+0800",
            "note 1",
        );
        let rsvp_10 = manager.reserve(rsvp_10).await.unwrap();
        let rsvp_10 = manager
            .change_status(Uuid::from_str(&rsvp_10.id).unwrap())
            .await
            .unwrap();
        let rsvp_10 = manager
            .delete(Uuid::from_str(&rsvp_10.id).unwrap())
            .await
            .unwrap();
        let err = manager
            .get(Uuid::from_str(&rsvp_10.id).unwrap())
            .await
            .unwrap_err();
        //TODO: parse error
        assert_eq!(
            err.to_string(),
            "DB error: no rows returned by a query that expected to return at least one row"
        );
    }
}
