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

use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::PgPool;
use std::str::FromStr;
use std::time::Duration;
use tracing::info;

// Re-export DatabaseConfig from config crate
pub use opengp_config::DatabaseConfig;

/// PostgreSQL-only connection pool
#[derive(Clone)]
pub struct DatabasePool(PgPool);

impl DatabasePool {
    pub fn size(&self) -> u32 {
        self.0.size()
    }

    pub fn as_postgres(&self) -> &PgPool {
        &self.0
    }

    pub fn kind(&self) -> &'static str {
        "postgres"
    }
}

/// Create a PostgreSQL connection pool
pub async fn create_pool(config: &DatabaseConfig) -> Result<DatabasePool, sqlx::Error> {
    info!("Creating PostgreSQL connection pool");
    info!("  Database URL: {}", config.url);
    info!("  Max connections: {}", config.max_connections);
    info!("  Min connections: {}", config.min_connections);

    let connect_options = PgConnectOptions::from_str(&config.url)?;

    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(Duration::from_secs(config.connect_timeout_secs))
        .idle_timeout(Duration::from_secs(config.idle_timeout_secs))
        .test_before_acquire(true)
        .connect_with(connect_options)
        .await?;

    info!("PostgreSQL connection pool created successfully");
    Ok(DatabasePool(pool))
}

/// Run database migrations
pub async fn run_migrations(pool: &DatabasePool) -> Result<(), sqlx::Error> {
    info!("Running database migrations");
    sqlx::migrate!("../../migrations_postgres").run(&pool.0).await?;
    info!("Database migrations completed successfully");
    Ok(())
}

/// Check database connection health
pub async fn health_check(pool: &DatabasePool) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1").execute(&pool.0).await?;
    Ok(())
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
