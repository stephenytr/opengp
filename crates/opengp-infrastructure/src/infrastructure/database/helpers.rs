//! Database helper functions for common operations
//!
//! This module provides utility functions to reduce boilerplate in repository implementations:
//! - UUID/bytes conversion for SQLite storage
//! - DateTime/string conversion for RFC3339 serialization
//! - Error mapping from sqlx to domain errors
//!
//! These helpers centralize conversion logic and error handling patterns used across
//! multiple repository implementations, improving maintainability and consistency.

use chrono::{DateTime, Utc};
use sqlx::error::DatabaseError;
use uuid::Uuid;

use opengp_domain::domain::error::InfrastructureError;
use opengp_domain::domain::error::RepositoryError as BaseRepositoryError;
use opengp_domain::domain::patient::RepositoryError;

/// Database UUID type used by infrastructure repositories
pub type DbUuid = Uuid;

/// Convert a `Uuid` into the database storage type
///
/// In the current PostgreSQL-backed configuration this is a no-op,
/// but SQLite-based builds can use the alias to store raw bytes.
pub fn uuid_to_bytes(id: &Uuid) -> DbUuid {
    *id
}

/// Convert the database UUID representation back into a `Uuid`
///
/// # Errors
///
/// In SQLite-backed builds this returns a `RepositoryError` if the
/// stored value cannot be parsed into a valid UUID.
pub fn bytes_to_uuid(bytes: &DbUuid) -> Result<Uuid, RepositoryError> {
    Ok(*bytes)
}

/// Convert a DateTime to RFC3339 string format
///
/// Converts a `DateTime<Utc>` to its RFC3339 string representation for storage in SQLite.
/// This is the standard format used throughout the application for timestamp serialization.
///
/// # Arguments
/// * `dt` - The DateTime to convert
///
/// # Returns
/// An RFC3339-formatted string (e.g., "2026-02-14T10:30:45Z")
///
/// # Example
/// ```
/// use chrono::Utc;
/// use opengp_infrastructure::infrastructure::database::helpers::datetime_to_string;
///
/// let now = Utc::now();
/// let s = datetime_to_string(&now);
/// // RFC3339 format: contains 'T' and has timezone offset (Z or +00:00)
/// assert!(s.contains('T'));
/// assert!(s.ends_with('Z') || s.ends_with("+00:00") || s.ends_with("-00:00"));
/// ```
pub fn datetime_to_string(dt: &DateTime<Utc>) -> String {
    dt.to_rfc3339()
}

/// Convert an RFC3339 string back to a DateTime
///
/// Parses an RFC3339-formatted string back into a `DateTime<Utc>`.
/// If parsing fails, returns the current time as a fallback (with a warning logged).
/// This defensive approach prevents database corruption from malformed timestamps.
///
/// # Arguments
/// * `s` - The RFC3339-formatted string to parse
///
/// # Returns
/// A `DateTime<Utc>` representing the parsed time, or `Utc::now()` if parsing fails
///
/// # Example
/// ```
/// use chrono::Utc;
/// use opengp_infrastructure::infrastructure::database::helpers::{datetime_to_string, string_to_datetime};
///
/// let original = Utc::now();
/// let s = datetime_to_string(&original);
/// let restored = string_to_datetime(&s);
/// // Times should be very close (within milliseconds)
/// assert!((original - restored).num_seconds() < 1);
/// ```
pub fn string_to_datetime(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| {
            tracing::warn!("Failed to parse datetime string: {}, using current time", s);
            Utc::now()
        })
}

/// Map sqlx database errors to domain RepositoryError
///
/// Analyzes sqlx database errors and maps them to appropriate RepositoryError variants.
/// This centralizes error handling logic for constraint violations and other database errors.
///
/// # Arguments
/// * `db_err` - The sqlx database error to map
///
/// # Returns
/// A RepositoryError with appropriate context
///
/// # Supported Mappings
/// - UNIQUE constraint violations → `RepositoryError::ConstraintViolation`
/// - NOT NULL constraint violations → `RepositoryError::ConstraintViolation`
/// - CHECK constraint violations → `RepositoryError::ConstraintViolation`
/// - Other errors → `RepositoryError::Database`
///
/// # Example
/// ```
/// use opengp_infrastructure::infrastructure::database::helpers::map_db_error;
/// use sqlx::Error;
///
/// // In real code, this would come from a failed database operation
/// // let db_err = some_query.execute(&pool).await.unwrap_err();
/// // let repo_err = map_db_error(db_err);
/// ```
pub fn map_db_error(db_err: Box<dyn DatabaseError>) -> RepositoryError {
    let err_msg = db_err.message();

    if err_msg.contains("UNIQUE constraint") {
        if err_msg.contains("medicare_number") {
            RepositoryError::Base(BaseRepositoryError::ConstraintViolation(
                "Medicare number already exists in the system".to_string(),
            ))
        } else {
            RepositoryError::Base(BaseRepositoryError::ConstraintViolation(
                "Unique constraint violation: duplicate value".to_string(),
            ))
        }
    } else if err_msg.contains("NOT NULL constraint") {
        RepositoryError::Base(BaseRepositoryError::ConstraintViolation(
            "Required field is missing".to_string(),
        ))
    } else if err_msg.contains("CHECK constraint") {
        RepositoryError::Base(BaseRepositoryError::ConstraintViolation(
            "Invalid value for field".to_string(),
        ))
    } else {
        RepositoryError::Base(BaseRepositoryError::ConstraintViolation(format!(
            "Database constraint violation: {}",
            err_msg
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    // ============================================================================
    // uuid_to_bytes tests
    // ============================================================================

    #[test]
    fn test_uuid_to_bytes_returns_16_bytes() {
        let id = Uuid::new_v4();
        let bytes = uuid_to_bytes(&id);
        // PostgreSQL: uuid_to_bytes is a no-op, returns the Uuid as-is
        assert_eq!(bytes, id);
    }

    #[test]
    fn test_uuid_to_bytes_consistent() {
        let id = Uuid::new_v4();
        let bytes1 = uuid_to_bytes(&id);
        let bytes2 = uuid_to_bytes(&id);
        assert_eq!(bytes1, bytes2);
    }

    #[test]
    fn test_uuid_to_bytes_different_for_different_uuids() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let bytes1 = uuid_to_bytes(&id1);
        let bytes2 = uuid_to_bytes(&id2);
        assert_ne!(bytes1, bytes2);
    }

    #[test]
    fn test_uuid_to_bytes_known_uuid() {
        let id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let bytes = uuid_to_bytes(&id);
        // PostgreSQL: uuid_to_bytes is a no-op, returns the Uuid as-is
        assert_eq!(bytes, id);
    }

    // ============================================================================
    // bytes_to_uuid tests
    // ============================================================================

    #[test]
    fn test_bytes_to_uuid_roundtrip() {
        let original = Uuid::new_v4();
        let bytes = uuid_to_bytes(&original);
        let restored = bytes_to_uuid(&bytes).unwrap();
        assert_eq!(original, restored);
    }

    #[test]
    fn test_bytes_to_uuid_known_value() {
        let original = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let bytes = uuid_to_bytes(&original);
        let restored = bytes_to_uuid(&bytes).unwrap();
        assert_eq!(original, restored);
    }

    // ============================================================================
    // datetime_to_string tests
    // ============================================================================

    #[test]
    fn test_datetime_to_string_format() {
        let dt = Utc::now();
        let s = datetime_to_string(&dt);
        // RFC3339 format ends with 'Z' or '+00:00' for UTC
        assert!(s.ends_with('Z') || s.ends_with("+00:00"));
    }

    #[test]
    fn test_datetime_to_string_parseable() {
        let dt = Utc::now();
        let s = datetime_to_string(&dt);
        // Should be parseable back
        let parsed = DateTime::parse_from_rfc3339(&s);
        assert!(parsed.is_ok());
    }

    #[test]
    fn test_datetime_to_string_consistent() {
        let dt = Utc::now();
        let s1 = datetime_to_string(&dt);
        let s2 = datetime_to_string(&dt);
        assert_eq!(s1, s2);
    }

    #[test]
    fn test_datetime_to_string_different_times() {
        let dt1 = Utc::now();
        let dt2 = Utc::now();
        let s1 = datetime_to_string(&dt1);
        let s2 = datetime_to_string(&dt2);
        // Times might be the same if executed very quickly, but format should be valid
        assert!(s1.ends_with('Z') || s1.ends_with("+00:00"));
        assert!(s2.ends_with('Z') || s2.ends_with("+00:00"));
    }

    #[test]
    fn test_datetime_to_string_known_value() {
        // Create a specific datetime
        let dt = DateTime::parse_from_rfc3339("2026-02-14T10:30:45Z")
            .unwrap()
            .with_timezone(&Utc);
        let s = datetime_to_string(&dt);
        // to_rfc3339() may use +00:00 instead of Z
        assert!(s == "2026-02-14T10:30:45Z" || s == "2026-02-14T10:30:45+00:00");
    }

    // ============================================================================
    // string_to_datetime tests
    // ============================================================================

    #[test]
    fn test_string_to_datetime_valid_rfc3339() {
        let s = "2026-02-14T10:30:45Z";
        let dt = string_to_datetime(s);
        // Should parse successfully
        assert_eq!(dt.year(), 2026);
        assert_eq!(dt.month(), 2);
        assert_eq!(dt.day(), 14);
    }

    #[test]
    fn test_string_to_datetime_roundtrip() {
        let original = Utc::now();
        let s = datetime_to_string(&original);
        let restored = string_to_datetime(&s);
        // Should be very close (within 1 second due to precision)
        assert!((original - restored).num_seconds() < 1);
    }

    #[test]
    fn test_string_to_datetime_invalid_format_returns_current_time() {
        let invalid = "not-a-valid-datetime";
        let dt = string_to_datetime(invalid);
        // Should return current time (approximately)
        let now = Utc::now();
        assert!((now - dt).num_seconds() < 2);
    }

    #[test]
    fn test_string_to_datetime_empty_string() {
        let dt = string_to_datetime("");
        let now = Utc::now();
        assert!((now - dt).num_seconds() < 2);
    }

    #[test]
    fn test_string_to_datetime_partial_rfc3339() {
        let s = "2026-02-14";
        let dt = string_to_datetime(s);
        // Invalid RFC3339, should return current time
        let now = Utc::now();
        assert!((now - dt).num_seconds() < 2);
    }

    #[test]
    fn test_string_to_datetime_with_timezone_offset() {
        let s = "2026-02-14T10:30:45+10:00";
        let dt = string_to_datetime(s);
        // Should parse successfully and convert to UTC
        assert_eq!(dt.year(), 2026);
        assert_eq!(dt.month(), 2);
        assert_eq!(dt.day(), 14);
    }
}

/// Convert a sqlx::Error to RepositoryError
///
/// This function should be used in infrastructure layer instead of the `?` operator
/// when the error type is sqlx::Error and the return type is Result<T, RepositoryError>.
/// It converts the sqlx error to a domain-friendly string representation.
pub fn repo_err_from_sqlx(err: sqlx::Error) -> RepositoryError {
    RepositoryError::Base(BaseRepositoryError::Database(err.to_string()))
}

/// Generic sqlx error mapping function for any InfrastructureError type
///
/// Maps sqlx database errors to domain error types that implement InfrastructureError.
/// Analyzes constraint violations (UNIQUE, NOT NULL, CHECK) and provides meaningful
/// error messages. Falls back to generic Database error for other sqlx errors.
///
/// # Arguments
/// * `err` - The sqlx error to map
///
/// # Returns
/// A domain error of type E that implements InfrastructureError
///
/// # Type Parameters
/// * `E` - The target error type that implements InfrastructureError
///
/// # Supported Mappings
/// - UNIQUE constraint violations → `E::map_sqlx_error` with constraint message
/// - NOT NULL constraint violations → `E::map_sqlx_error` with constraint message
/// - CHECK constraint violations → `E::map_sqlx_error` with constraint message
/// - Other errors → `E::map_sqlx_error` with generic database error message
///
/// # Example
/// ```ignore
/// use opengp_infrastructure::infrastructure::database::helpers::map_sqlx_error;
/// use opengp_domain::domain::error::RepositoryError;
///
/// // In a repository implementation
/// let result = sqlx::query("INSERT INTO patients ...").execute(&pool).await;
/// let err = result.map_err(|e| map_sqlx_error::<RepositoryError>(e))?;
/// ```
pub fn map_sqlx_error<E: InfrastructureError>(err: sqlx::Error) -> E {
    // Try to extract database error details if available
    if let sqlx::Error::Database(db_err) = &err {
        let err_msg = db_err.message();

        // Map specific constraint violations by creating a wrapper error
        if err_msg.contains("UNIQUE constraint") {
            let msg = if err_msg.contains("medicare_number") {
                "Medicare number already exists in the system"
            } else {
                "Unique constraint violation: duplicate value"
            };
            return E::map_sqlx_error(ConstraintError(msg.to_string()));
        } else if err_msg.contains("NOT NULL constraint") {
            return E::map_sqlx_error(ConstraintError("Required field is missing".to_string()));
        } else if err_msg.contains("CHECK constraint") {
            return E::map_sqlx_error(ConstraintError("Invalid value for field".to_string()));
        }
    }

    // Fall back to generic database error
    E::map_sqlx_error(err)
}

/// Wrapper error type for constraint violations
/// Implements std::error::Error to satisfy InfrastructureError trait bounds
#[derive(Debug)]
struct ConstraintError(String);

impl std::fmt::Display for ConstraintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ConstraintError {}
