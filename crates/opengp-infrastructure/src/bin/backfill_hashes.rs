use std::env;

use opengp_infrastructure::infrastructure::crypto::EncryptionService;
use sqlx::{postgres::PgPoolOptions, FromRow};
use thiserror::Error;
use tracing::{error, info, warn};
use uuid::Uuid;

const BATCH_SIZE: i64 = 100;
const LOG_INTERVAL: u64 = 1_000;

#[derive(Debug, FromRow)]
struct PatientBackfillRow {
    id: Uuid,
    medicare_number: Vec<u8>,
}

#[derive(Debug, Error)]
enum BackfillError {
    #[error("DATABASE_URL environment variable is required")]
    MissingDatabaseUrl,

    #[error("Failed to initialize crypto service: {0}")]
    Crypto(#[from] opengp_infrastructure::infrastructure::crypto::CryptoError),

    #[error("Database operation failed: {0}")]
    Database(#[from] sqlx::Error),
}

#[tokio::main]
async fn main() -> Result<(), BackfillError> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = env::var("DATABASE_URL").map_err(|_| BackfillError::MissingDatabaseUrl)?;
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    let crypto = EncryptionService::new()?;

    let mut total_updated: u64 = 0;

    loop {
        let rows = sqlx::query_as::<_, PatientBackfillRow>(
            r#"
            SELECT id, medicare_number
            FROM patients
            WHERE medicare_search_hash IS NULL
              AND medicare_number IS NOT NULL
            ORDER BY id
            LIMIT $1
            "#,
        )
        .bind(BATCH_SIZE)
        .fetch_all(&pool)
        .await?;

        if rows.is_empty() {
            break;
        }

        for row in rows {
            let medicare = decode_medicare_number(&crypto, row.id, row.medicare_number);

            let Some(medicare_number) = medicare else {
                continue;
            };

            let medicare_hash = crypto.hash_for_search(&medicare_number);

            let result = sqlx::query(
                r#"
                UPDATE patients
                SET medicare_search_hash = $1
                WHERE id = $2
                  AND medicare_search_hash IS NULL
                "#,
            )
            .bind(medicare_hash)
            .bind(row.id)
            .execute(&pool)
            .await?;

            if result.rows_affected() == 1 {
                total_updated += 1;
                if total_updated.is_multiple_of(LOG_INTERVAL) {
                    info!(updated = total_updated, "Backfill progress");
                }
            }
        }
    }

    info!(
        updated = total_updated,
        "Medicare search hash backfill complete"
    );
    Ok(())
}

fn decode_medicare_number(
    crypto: &EncryptionService,
    patient_id: Uuid,
    medicare_number: Vec<u8>,
) -> Option<String> {
    match crypto.decrypt(&medicare_number) {
        Ok(plaintext) => Some(plaintext),
        Err(err) => {
            warn!(
                patient_id = %patient_id,
                error = %err,
                "Decryption failed, attempting plaintext fallback"
            );

            match String::from_utf8(medicare_number) {
                Ok(plaintext) => Some(plaintext),
                Err(utf8_err) => {
                    error!(
                        patient_id = %patient_id,
                        error = %utf8_err,
                        "Skipping patient: medicare_number is neither decryptable nor UTF-8"
                    );
                    None
                }
            }
        }
    }
}
