use chrono::{NaiveTime, Timelike, Utc};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::infrastructure::database::helpers::uuid_to_bytes;
use opengp_domain::domain::user::WorkingHours;

/// Seed working hours for all practitioners in the database.
///
/// Creates working hours entries for:
/// - Monday-Friday (days 0-4): 08:00 - 17:00
/// - Saturday (day 5): 09:00 - 13:00
/// - Sunday (day 6): Not working (no entry)
///
/// The function is idempotent and will skip practitioners that already have working hours defined.
pub async fn seed_working_hours(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id FROM users WHERE (role = 'Doctor' OR role = 'Nurse') AND is_active = TRUE",
    )
    .fetch_all(pool)
    .await?;

    tracing::info!("Seeding working hours for {} practitioners", rows.len());

    for row in rows {
        let practitioner_id_bytes: Vec<u8> = row.get("id");

        let existing_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM working_hours WHERE practitioner_id = ?",
        )
        .bind(&practitioner_id_bytes)
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
                practitioner_id: bytes_to_uuid(&practitioner_id_bytes).map_err(|e| {
                    sqlx::Error::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
                })?,
                day_of_week: day_of_week as u8,
                start_time,
                end_time,
                is_active: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            let id_bytes = uuid_to_bytes(&working_hours.id);
            let start_time_str = working_hours.start_time.format("%H:%M:%S").to_string();
            let end_time_str = working_hours.end_time.format("%H:%M:%S").to_string();
            let created_at_str =
                format!("{}", working_hours.created_at.format("%Y-%m-%d %H:%M:%S"));
            let updated_at_str =
                format!("{}", working_hours.updated_at.format("%Y-%m-%d %H:%M:%S"));

            sqlx::query(
                r#"
                INSERT INTO working_hours (
                    id, practitioner_id, day_of_week,
                    start_time, end_time,
                    is_active,
                    created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(&id_bytes)
            .bind(&practitioner_id_bytes)
            .bind(day_of_week as i64)
            .bind(&start_time_str)
            .bind(&end_time_str)
            .bind(true)
            .bind(&created_at_str)
            .bind(&updated_at_str)
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

fn bytes_to_uuid(bytes: &[u8]) -> Result<Uuid, String> {
    if bytes.len() != 16 {
        return Err("Invalid UUID bytes length".to_string());
    }
    let mut uuid_bytes = [0u8; 16];
    uuid_bytes.copy_from_slice(bytes);
    Ok(Uuid::from_bytes(uuid_bytes))
}
