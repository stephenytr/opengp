//! Database infrastructure
//!
//! This module manages database connections, connection pooling, and migrations.

pub mod helpers;
pub mod query;
pub mod repositories;

pub use helpers::*;
pub use query::*;

pub mod mocks;

#[cfg(test)]
pub mod test_utils;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::str::FromStr;
use std::time::Duration;
use tracing::info;

// Re-export DatabaseConfig from config crate
pub use opengp_config::DatabaseConfig;

/// Create a configured database connection pool
///
/// # Arguments
/// * `config` - Database configuration
///
/// # Returns
/// * `Ok(SqlitePool)` - Successfully created connection pool
/// * `Err(sqlx::Error)` - Failed to create pool or connect to database
///
/// # Example
/// ```no_run
/// use opengp_infrastructure::infrastructure::database::{DatabaseConfig, create_pool};
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), sqlx::Error> {
/// let config = DatabaseConfig::default();
/// let pool = create_pool(&config).await?;
/// # Ok(())
/// # }
/// ```
pub async fn create_pool(config: &DatabaseConfig) -> Result<SqlitePool, sqlx::Error> {
    info!("Creating database connection pool");
    info!("  Database URL: {}", config.url);
    info!("  Max connections: {}", config.max_connections);
    info!("  Min connections: {}", config.min_connections);

    // Parse connection options
    let connect_options = SqliteConnectOptions::from_str(&config.url)?
        .create_if_missing(true)
        .busy_timeout(Duration::from_secs(30));

    // Create pool with configuration
    let pool = SqlitePoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(Duration::from_secs(config.connect_timeout_secs))
        .idle_timeout(Duration::from_secs(config.idle_timeout_secs))
        .test_before_acquire(true) // Verify connections before use
        .connect_with(connect_options)
        .await?;

    info!("Database connection pool created successfully");

    Ok(pool)
}

/// Run database migrations
///
/// # Arguments
/// * `pool` - Database connection pool
///
/// # Returns
/// * `Ok(())` - Migrations applied successfully
/// * `Err(sqlx::Error)` - Migration failed
pub async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    info!("Running database migrations");

    // Force recompilation by including all migrations
    sqlx::migrate!("./migrations").run(pool).await?;

    // Workaround: Manually ensure working_hours table exists
    // The sqlx::migrate! macro sometimes doesn't pick up new migration files at compile time
    // This fallback ensures the table is created if it doesn't exist
    ensure_working_hours_table(pool).await?;
    ensure_sessions_have_token(pool).await?;

    info!("Database migrations completed successfully");

    Ok(())
}

/// Ensure working_hours table exists (fallback for sqlx macro issue)
///
/// This is a workaround for the sqlx::migrate! macro not picking up new migration files.
/// It's idempotent and safe to run multiple times.
async fn ensure_working_hours_table(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Check if table already exists
    let table_exists: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='working_hours'",
    )
    .fetch_one(pool)
    .await?;

    if table_exists.0 > 0 {
        info!("working_hours table already exists, skipping creation");
        return Ok(());
    }

    info!("Creating working_hours table (sqlx macro fallback)");

    // Create the working_hours table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS working_hours (
            id BLOB PRIMARY KEY,
            practitioner_id BLOB NOT NULL,
            day_of_week INTEGER NOT NULL CHECK(day_of_week BETWEEN 0 AND 6),
            start_time TEXT NOT NULL,
            end_time TEXT NOT NULL,
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (practitioner_id) REFERENCES users(id),
            UNIQUE(practitioner_id, day_of_week)
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create indexes
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_working_hours_practitioner ON working_hours(practitioner_id)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_working_hours_day ON working_hours(practitioner_id, day_of_week)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_working_hours_active ON working_hours(is_active)")
        .execute(pool)
        .await?;

    info!("working_hours table created successfully");

    Ok(())
}

async fn ensure_sessions_have_token(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let column_exists: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM pragma_table_info('sessions') WHERE name='token'")
            .fetch_one(pool)
            .await?;

    if column_exists.0 > 0 {
        return Ok(());
    }

    sqlx::query("ALTER TABLE sessions ADD COLUMN token TEXT")
        .execute(pool)
        .await?;

    sqlx::query(
        "UPDATE sessions SET token = lower(hex(randomblob(32))) WHERE token IS NULL OR token = ''",
    )
    .execute(pool)
    .await?;

    sqlx::query("CREATE UNIQUE INDEX IF NOT EXISTS idx_sessions_token ON sessions(token)")
        .execute(pool)
        .await?;

    Ok(())
}

/// Check database connection health
///
/// # Arguments
/// * `pool` - Database connection pool
///
/// # Returns
/// * `Ok(())` - Database is healthy
/// * `Err(sqlx::Error)` - Database is unhealthy
pub async fn health_check(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1").execute(pool).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_pool_in_memory() {
        let config = DatabaseConfig {
            url: "sqlite::memory:".to_string(),
            ..Default::default()
        };

        let pool = create_pool(&config).await;
        assert!(pool.is_ok());
    }

    #[tokio::test]
    async fn test_health_check() {
        let config = DatabaseConfig {
            url: "sqlite::memory:".to_string(),
            ..Default::default()
        };

        let pool = create_pool(&config).await.unwrap();
        let result = health_check(&pool).await;
        assert!(result.is_ok());
    }
}

// Re-export domain error types for infrastructure use
pub use opengp_domain::domain::error::RepositoryError;

/// Convert sqlx::Error to domain RepositoryError
pub fn sqlx_to_repository_error(err: sqlx::Error) -> RepositoryError {
    RepositoryError::Database(err.to_string())
}

/// Convert sqlx::Error to domain AuditRepositoryError
pub fn sqlx_to_audit_error(err: sqlx::Error) -> opengp_domain::domain::audit::AuditRepositoryError {
    use opengp_domain::domain::error::RepositoryError as Base;
    opengp_domain::domain::audit::AuditRepositoryError::Base(Base::Database(err.to_string()))
}

/// Convert sqlx::Error to domain clinical RepositoryError
pub fn sqlx_to_clinical_error(
    err: sqlx::Error,
) -> opengp_domain::domain::clinical::RepositoryError {
    use opengp_domain::domain::error::RepositoryError as Base;
    opengp_domain::domain::clinical::RepositoryError::Base(Base::Database(err.to_string()))
}

/// Convert sqlx::Error to domain patient RepositoryError
pub fn sqlx_to_patient_error(err: sqlx::Error) -> opengp_domain::domain::patient::RepositoryError {
    use opengp_domain::domain::error::RepositoryError as Base;
    opengp_domain::domain::patient::RepositoryError::Base(Base::Database(err.to_string()))
}

/// Convert sqlx::Error to domain user RepositoryError
pub fn sqlx_to_user_error(err: sqlx::Error) -> opengp_domain::domain::user::RepositoryError {
    opengp_domain::domain::user::RepositoryError::Database(err.to_string())
}

/// Convert sqlx::Error to domain appointment RepositoryError
pub fn sqlx_to_appointment_error(
    err: sqlx::Error,
) -> opengp_domain::domain::appointment::RepositoryError {
    opengp_domain::domain::appointment::RepositoryError::Database(err.to_string())
}
