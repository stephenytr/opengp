use chrono::{NaiveTime, Timelike, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use opengp_domain::domain::user::WorkingHours;

pub async fn seed_working_hours(pool: &PgPool) -> Result<(), sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id FROM users WHERE (role = 'Doctor' OR role = 'Nurse') AND is_active = TRUE",
    )
    .fetch_all(pool)
    .await?;

    tracing::info!("Seeding working hours for {} practitioners", rows.len());

    for row in rows {
        let practitioner_id: Uuid = row.get("id");

        let existing_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM working_hours WHERE practitioner_id = $1",
        )
        .bind(practitioner_id)
        .fetch_one(pool)
        .await?;

        if existing_count > 0 {
            tracing::debug!(
                "Practitioner already has {} working hours entries, skipping",
                existing_count
            );
            continue;
        }

        for day_of_week in 0..6 {
            // SAFETY: hours/minutes are compile-time constants that are always valid
            #[allow(clippy::unwrap_used)]
            let (start_time, end_time) = match day_of_week {
                0..=4 => (
                    NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
                    NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
                ),
                5 => (
                    NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
                    NaiveTime::from_hms_opt(13, 0, 0).unwrap(),
                ),
                _ => unreachable!(),
            };

            let working_hours = WorkingHours {
                id: Uuid::new_v4(),
                practitioner_id,
                day_of_week: day_of_week as u8,
                start_time,
                end_time,
                is_active: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            sqlx::query(
                r#"
                INSERT INTO working_hours (
                    id, practitioner_id, day_of_week,
                    start_time, end_time,
                    is_active,
                    created_at, updated_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
            )
            .bind(working_hours.id)
            .bind(practitioner_id)
            .bind(day_of_week as i64)
            .bind(working_hours.start_time)
            .bind(working_hours.end_time)
            .bind(true)
            .bind(working_hours.created_at)
            .bind(working_hours.updated_at)
            .execute(pool)
            .await?;

            tracing::debug!(
                "Created working hours for day {} ({}:00 - {}:00)",
                day_of_week,
                start_time.hour(),
                end_time.hour()
            );
        }
    }

    tracing::info!("Working hours seeding completed successfully");
    Ok(())
}
