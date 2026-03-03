//! Test utilities for repository testing
//!
//! This module provides helper functions for creating test fixtures and database pools
//! for use in integration tests.

use chrono::{Duration, NaiveDate, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

use opengp_domain::domain::appointment::{Appointment, AppointmentType};
use opengp_domain::domain::patient::{Address, Gender, NewPatientData, Patient};
use crate::infrastructure::database::{create_pool, run_migrations, DatabaseConfig};

/// Create an in-memory SQLite pool for testing
///
/// This function creates a new in-memory SQLite database, runs all migrations,
/// and returns a connection pool ready for testing.
///
/// # Returns
/// * `Ok(SqlitePool)` - Successfully created and initialized pool
/// * `Err(sqlx::Error)` - Failed to create pool or run migrations
///
/// # Example
/// ```no_run
/// use opengp_infrastructure::infrastructure::database::test_utils::create_test_pool;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), sqlx::Error> {
/// let pool = create_test_pool().await?;
/// // Use pool for testing
/// # Ok(())
/// # }
/// ```
pub async fn create_test_pool() -> Result<SqlitePool, sqlx::Error> {
    let config = DatabaseConfig {
        url: "sqlite::memory:".to_string(),
        max_connections: 5,
        min_connections: 1,
        connect_timeout_secs: 10,
        idle_timeout_secs: 60,
    };

    let pool = create_pool(&config).await?;
    run_migrations(&pool).await?;

    Ok(pool)
}

/// Create a test patient fixture with default values
///
/// Returns a valid Patient with sensible test data. All required fields are populated.
/// The patient is created with:
/// - First name: "John"
/// - Last name: "Smith"
/// - Date of birth: 1980-01-15 (making them ~46 years old)
/// - Gender: Male
/// - Active status: true
/// - Default Australian address
///
/// # Returns
/// A valid Patient instance ready for testing
///
/// # Example
/// ```
/// use opengp_infrastructure::infrastructure::database::test_utils::create_test_patient;
///
/// let patient = create_test_patient();
/// assert_eq!(patient.first_name, "John");
/// assert_eq!(patient.last_name, "Smith");
/// assert!(patient.is_active);
/// ```
pub fn create_test_patient() -> Patient {
    let data = NewPatientData {
        ihi: Some("8003608166701751".to_string()),
        medicare_number: Some("2123456789".to_string()),
        medicare_irn: Some(1),
        medicare_expiry: None,
        title: Some("Mr".to_string()),
        first_name: "John".to_string(),
        middle_name: Some("Michael".to_string()),
        last_name: "Smith".to_string(),
        preferred_name: None,
        date_of_birth: NaiveDate::from_ymd_opt(1980, 1, 15).unwrap(),
        gender: Gender::Male,
        address: Address {
            line1: Some("123 Main Street".to_string()),
            line2: None,
            suburb: Some("Sydney".to_string()),
            state: Some("NSW".to_string()),
            postcode: Some("2000".to_string()),
            country: "Australia".to_string(),
        },
        phone_home: Some("(02) 9555 1234".to_string()),
        phone_mobile: Some("0412 345 678".to_string()),
        email: Some("john.smith@example.com".to_string()),
        emergency_contact: None,
        concession_type: None,
        concession_number: None,
        preferred_language: Some("English".to_string()),
        interpreter_required: Some(false),
        aboriginal_torres_strait_islander: None,
    };

    Patient::new(
        data.first_name,
        data.last_name,
        data.date_of_birth,
        data.gender,
        data.ihi,
        data.medicare_number,
        data.medicare_irn,
        data.medicare_expiry,
        data.title,
        data.middle_name,
        data.preferred_name,
        data.address,
        data.phone_home,
        data.phone_mobile,
        data.email,
        data.emergency_contact,
        data.concession_type,
        data.concession_number,
        data.preferred_language,
        data.interpreter_required,
        data.aboriginal_torres_strait_islander,
    )
    .expect("Failed to create test patient")
}

/// Create a test appointment fixture with default values
///
/// Returns a valid Appointment with sensible test data. The appointment is created with:
/// - Random patient and practitioner IDs
/// - Start time: 2 hours from now
/// - Duration: 15 minutes (standard appointment)
/// - Type: Standard consultation
/// - Status: Scheduled
/// - Not urgent, not confirmed, no reminder sent
///
/// # Returns
/// A valid Appointment instance ready for testing
///
/// # Example
/// ```
/// use opengp_infrastructure::infrastructure::database::test_utils::create_test_appointment;
///
/// let appointment = create_test_appointment();
/// assert_eq!(appointment.appointment_type, opengp_domain::domain::appointment::model::AppointmentType::Standard);
/// assert!(!appointment.is_urgent);
/// ```
pub fn create_test_appointment() -> Appointment {
    let start_time = Utc::now() + Duration::hours(2);
    let duration = Duration::minutes(15);

    Appointment::new(
        Uuid::new_v4(),
        Uuid::new_v4(),
        start_time,
        duration,
        AppointmentType::Standard,
        Some(Uuid::new_v4()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_test_pool() {
        let pool = create_test_pool().await;
        assert!(pool.is_ok());

        let pool = pool.unwrap();
        let result = sqlx::query("SELECT 1").execute(&pool).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_test_patient() {
        let patient = create_test_patient();

        assert_eq!(patient.first_name, "John");
        assert_eq!(patient.last_name, "Smith");
        assert_eq!(patient.gender, Gender::Male);
        assert!(patient.is_active);
        assert!(!patient.is_deceased);
        assert!(patient.age() >= 45);
        assert!(patient.medicare_number.is_some());
        assert!(patient.ihi.is_some());
    }

    #[test]
    fn test_create_test_patient_has_valid_address() {
        let patient = create_test_patient();

        assert_eq!(patient.address.suburb, Some("Sydney".to_string()));
        assert_eq!(patient.address.state, Some("NSW".to_string()));
        assert_eq!(patient.address.postcode, Some("2000".to_string()));
        assert_eq!(patient.address.country, "Australia");
    }

    #[test]
    fn test_create_test_patient_has_contact_info() {
        let patient = create_test_patient();

        assert!(patient.phone_home.is_some());
        assert!(patient.phone_mobile.is_some());
        assert!(patient.email.is_some());
    }

    #[test]
    fn test_create_test_appointment() {
        let appointment = create_test_appointment();

        assert_eq!(appointment.appointment_type, AppointmentType::Standard);
        assert!(!appointment.is_urgent);
        assert!(!appointment.confirmed);
        assert!(!appointment.reminder_sent);
        assert_eq!(appointment.duration_minutes(), 15);
        assert!(!appointment.is_past());
    }

    #[test]
    fn test_create_test_appointment_has_valid_times() {
        let appointment = create_test_appointment();

        assert!(appointment.start_time > Utc::now());
        assert!(appointment.end_time > appointment.start_time);
    }

    #[test]
    fn test_create_test_appointment_has_audit_fields() {
        let appointment = create_test_appointment();

        assert!(appointment.created_by.is_some());
        assert!(appointment.created_at <= Utc::now());
    }
}
