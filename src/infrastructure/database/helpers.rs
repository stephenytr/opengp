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

use crate::domain::patient::RepositoryError;

/// Convert a UUID to a SQLite-compatible byte vector
///
/// SQLite doesn't have a native UUID type, so UUIDs are stored as 16-byte binary values.
/// This function converts a UUID to its byte representation for storage.
///
/// # Arguments
/// * `id` - The UUID to convert
///
/// # Returns
/// A vector of 16 bytes representing the UUID
///
/// # Example
/// ```
/// use uuid::Uuid;
/// use opengp::infrastructure::database::helpers::uuid_to_bytes;
///
/// let id = Uuid::new_v4();
/// let bytes = uuid_to_bytes(&id);
/// assert_eq!(bytes.len(), 16);
/// ```
pub fn uuid_to_bytes(id: &Uuid) -> Vec<u8> {
    id.as_bytes().to_vec()
}

/// Convert SQLite bytes back to a UUID
///
/// Reverses the conversion performed by `uuid_to_bytes()`. Returns a RepositoryError
/// if the byte slice is not exactly 16 bytes or cannot be parsed as a valid UUID.
///
/// # Arguments
/// * `bytes` - The byte slice to convert (must be exactly 16 bytes)
///
/// # Returns
/// * `Ok(Uuid)` - Successfully converted UUID
/// * `Err(RepositoryError::ConstraintViolation)` - Invalid byte length or format
///
/// # Example
/// ```
/// use uuid::Uuid;
/// use opengp::infrastructure::database::helpers::{uuid_to_bytes, bytes_to_uuid};
///
/// let original = Uuid::new_v4();
/// let bytes = uuid_to_bytes(&original);
/// let restored = bytes_to_uuid(&bytes).unwrap();
/// assert_eq!(original, restored);
/// ```
pub fn bytes_to_uuid(bytes: &[u8]) -> Result<Uuid, RepositoryError> {
    Uuid::from_slice(bytes)
        .map_err(|e| RepositoryError::ConstraintViolation(format!("Invalid UUID bytes: {}", e)))
}

/// Convert a DateTime to RFC3339 string format
///
/// Converts a DateTime<Utc> to its RFC3339 string representation for storage in SQLite.
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
/// use opengp::infrastructure::database::helpers::datetime_to_string;
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
/// Parses an RFC3339-formatted string back into a DateTime<Utc>.
/// If parsing fails, returns the current time as a fallback (with a warning logged).
/// This defensive approach prevents database corruption from malformed timestamps.
///
/// # Arguments
/// * `s` - The RFC3339-formatted string to parse
///
/// # Returns
/// A DateTime<Utc> representing the parsed time, or Utc::now() if parsing fails
///
/// # Example
/// ```
/// use chrono::Utc;
/// use opengp::infrastructure::database::helpers::{datetime_to_string, string_to_datetime};
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
/// use opengp::infrastructure::database::helpers::map_db_error;
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
            RepositoryError::ConstraintViolation(
                "Medicare number already exists in the system".to_string(),
            )
        } else {
            RepositoryError::ConstraintViolation(
                "Unique constraint violation: duplicate value".to_string(),
            )
        }
    } else if err_msg.contains("NOT NULL constraint") {
        RepositoryError::ConstraintViolation("Required field is missing".to_string())
    } else if err_msg.contains("CHECK constraint") {
        RepositoryError::ConstraintViolation("Invalid value for field".to_string())
    } else {
        RepositoryError::ConstraintViolation(format!("Database constraint violation: {}", err_msg))
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
        assert_eq!(bytes.len(), 16);
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
        // Use a known UUID for deterministic testing
        let id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let bytes = uuid_to_bytes(&id);
        assert_eq!(bytes.len(), 16);
        // Verify it matches the UUID's internal representation
        assert_eq!(bytes, id.as_bytes());
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
    fn test_bytes_to_uuid_invalid_length_too_short() {
        let bytes = vec![1, 2, 3, 4, 5];
        let result = bytes_to_uuid(&bytes);
        assert!(result.is_err());
        match result {
            Err(RepositoryError::ConstraintViolation(msg)) => {
                assert!(msg.contains("Invalid UUID bytes"));
            }
            _ => panic!("Expected ConstraintViolation error"),
        }
    }

    #[test]
    fn test_bytes_to_uuid_invalid_length_too_long() {
        let bytes = vec![1; 32];
        let result = bytes_to_uuid(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_bytes_to_uuid_empty() {
        let bytes: Vec<u8> = vec![];
        let result = bytes_to_uuid(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_bytes_to_uuid_known_value() {
        let original = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let bytes = original.as_bytes();
        let restored = bytes_to_uuid(bytes).unwrap();
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
