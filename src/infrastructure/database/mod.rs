//! Database infrastructure
//!
//! This module manages database connections, connection pooling, and migrations.

pub mod repositories;

use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::str::FromStr;
use std::time::Duration;
use tracing::info;

/// Database pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout_secs: u64,
    pub idle_timeout_secs: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "sqlite:opengp.db".to_string(),
            max_connections: 10,
            min_connections: 2,
            connect_timeout_secs: 30,
            idle_timeout_secs: 600, // 10 minutes
        }
    }
}

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
/// use opengp::infrastructure::database::{DatabaseConfig, create_pool};
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

    sqlx::migrate!("./migrations")
        .run(pool)
        .await?;

    info!("Database migrations completed successfully");

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
    sqlx::query("SELECT 1")
        .execute(pool)
        .await?;

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
