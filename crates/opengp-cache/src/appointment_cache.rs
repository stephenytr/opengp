use chrono::NaiveDate;
use uuid::Uuid;

use crate::error::CacheError;
use crate::service::CacheServiceImpl;
use opengp_domain::domain::appointment::CalendarAppointment;

/// TTL for appointment slot caching: 120 seconds (2 minutes)
const APPOINTMENT_CACHE_TTL_SECS: u64 = 120;

/// Get appointment slots from cache for a specific practice and date
///
/// # Arguments
/// * `cache` - The cache service implementation
/// * `prac_id` - Practice UUID
/// * `date` - Date to query for
///
/// # Returns
/// * `Ok(Some(Vec<CalendarAppointment>))` if cache hit
/// * `Ok(None)` if cache miss
/// * `Err(CacheError)` if cache operation fails
pub async fn get_appointment_slots(
    cache: &CacheServiceImpl,
    prac_id: Uuid,
    date: NaiveDate,
) -> Result<Option<Vec<CalendarAppointment>>, CacheError> {
    let key = format!("appt:{}:{}", prac_id, date);
    cache.get(&key).await
}

/// Set appointment slots in cache for a specific practice and date
///
/// # Arguments
/// * `cache` - The cache service implementation
/// * `prac_id` - Practice UUID
/// * `date` - Date to cache for
/// * `slots` - Appointment slots to cache
/// * `ttl` - Time to live in seconds (overrides default TTL)
///
/// # Returns
/// * `Ok(())` if successful
/// * `Err(CacheError)` if cache operation fails
pub async fn set_appointment_slots(
    cache: &CacheServiceImpl,
    prac_id: Uuid,
    date: NaiveDate,
    slots: &[CalendarAppointment],
    ttl: u64,
) -> Result<(), CacheError> {
    let key = format!("appt:{}:{}", prac_id, date);
    cache.set(&key, &slots.to_vec(), Some(ttl)).await
}

/// Invalidate cached appointment slots for a specific practice and date
///
/// # Arguments
/// * `cache` - The cache service implementation
/// * `prac_id` - Practice UUID
/// * `date` - Date to invalidate cache for
///
/// # Returns
/// * `Ok(())` if successful
/// * `Err(CacheError)` if cache operation fails
pub async fn invalidate_appointment_slots(
    cache: &CacheServiceImpl,
    prac_id: Uuid,
    date: NaiveDate,
) -> Result<(), CacheError> {
    let key = format!("appt:{}:{}", prac_id, date);
    cache.invalidate(&key).await
}

/// Invalidate all cached appointment slots for a specific practice
///
/// # Arguments
/// * `cache` - The cache service implementation
/// * `prac_id` - Practice UUID
///
/// # Returns
/// * `Ok(count)` - number of keys deleted
/// * `Err(CacheError)` if cache operation fails
pub async fn invalidate_all_practice_appointments(
    cache: &CacheServiceImpl,
    prac_id: Uuid,
) -> Result<u64, CacheError> {
    let pattern = format!("appt:{}:*", prac_id);
    cache.invalidate_pattern(&pattern).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    fn create_mock_appointment(patient_name: &str) -> CalendarAppointment {
        CalendarAppointment {
            id: Uuid::new_v4(),
            patient_id: Uuid::new_v4(),
            practitioner_id: Uuid::new_v4(),
            patient_name: patient_name.to_string(),
            start_time: Utc::now(),
            end_time: Utc::now() + Duration::minutes(15),
            appointment_type: opengp_domain::domain::appointment::AppointmentType::Standard,
            status: opengp_domain::domain::appointment::AppointmentStatus::Scheduled,
            is_urgent: false,
            slot_span: 1,
            reason: Some("Test appointment".to_string()),
            notes: None,
        }
    }

    #[test]
    fn test_key_format_single_appointment() {
        let prac_id = Uuid::new_v4();
        let date = NaiveDate::from_ymd_opt(2026, 3, 24).unwrap();
        let key = format!("appt:{}:{}", prac_id, date);

        assert!(key.starts_with("appt:"));
        assert!(key.contains(prac_id.to_string().as_str()));
        assert!(key.contains("2026-03-24"));
    }

    #[test]
    fn test_pattern_format_practice() {
        let prac_id = Uuid::new_v4();
        let pattern = format!("appt:{}:*", prac_id);

        assert!(pattern.starts_with("appt:"));
        assert!(pattern.contains(prac_id.to_string().as_str()));
        assert!(pattern.ends_with(":*"));
    }

    #[test]
    fn test_cache_ttl_constant() {
        assert_eq!(APPOINTMENT_CACHE_TTL_SECS, 120);
    }

    #[test]
    fn test_mock_appointment_creation() {
        let appt = create_mock_appointment("John Doe");
        assert_eq!(appt.patient_name, "John Doe");
        assert_eq!(appt.slot_span, 1);
        assert_eq!(appt.is_urgent, false);
    }
}
