//! Patient data cache integration
//!
//! Provides high-level functions for caching and retrieving patient data.
//! Uses Redis with a consistent key pattern and TTL strategy.

use opengp_domain::domain::patient::Patient;
use uuid::Uuid;

use crate::error::CacheError;
use crate::service::CacheServiceImpl;

/// Build a cache key for a patient by ID
fn patient_cache_key(id: Uuid) -> String {
    format!("patient:{}", id)
}

/// Get a patient from cache by ID
///
/// # Arguments
/// * `cache` - Cache service instance
/// * `id` - Patient UUID
///
/// # Returns
/// * `Ok(Some(Patient))` if patient is found in cache
/// * `Ok(None)` if patient is not in cache
/// * `Err(CacheError)` if cache operation fails (circuit breaker, serialization, etc.)
pub async fn get_patient_by_id(
    cache: &CacheServiceImpl,
    id: Uuid,
) -> Result<Option<Patient>, CacheError> {
    let key = patient_cache_key(id);
    cache.get::<Patient>(&key).await
}

/// Set a patient in cache
///
/// # Arguments
/// * `cache` - Cache service instance
/// * `patient` - Patient to cache
/// * `ttl` - Time to live in seconds
///
/// # Returns
/// * `Ok(())` if successful
/// * `Err(CacheError)` if cache operation fails
pub async fn set_patient(
    cache: &CacheServiceImpl,
    patient: &Patient,
    ttl: u64,
) -> Result<(), CacheError> {
    let key = patient_cache_key(patient.id);
    cache.set(&key, patient, Some(ttl)).await
}

/// Set a patient in cache with default TTL
///
/// # Arguments
/// * `cache` - Cache service instance
/// * `patient` - Patient to cache
///
/// # Returns
/// * `Ok(())` if successful
/// * `Err(CacheError)` if cache operation fails
pub async fn set_patient_default_ttl(
    cache: &CacheServiceImpl,
    patient: &Patient,
) -> Result<(), CacheError> {
    set_patient(cache, patient, cache.patient_ttl_secs()).await
}

/// Invalidate (delete) a patient from cache
///
/// # Arguments
/// * `cache` - Cache service instance
/// * `id` - Patient UUID to invalidate
///
/// # Returns
/// * `Ok(())` if successful (even if patient was not in cache)
/// * `Err(CacheError)` if cache operation fails
pub async fn invalidate_patient(cache: &CacheServiceImpl, id: Uuid) -> Result<(), CacheError> {
    let key = patient_cache_key(id);
    cache.invalidate(&key).await
}

/// Invalidate all patient cache entries matching a pattern
///
/// # Arguments
/// * `cache` - Cache service instance
/// * `pattern` - Pattern to match (e.g., "patient:*")
///
/// # Returns
/// * `Ok(count)` - Number of keys deleted
/// * `Err(CacheError)` if cache operation fails
pub async fn invalidate_patient_pattern(
    cache: &CacheServiceImpl,
    pattern: &str,
) -> Result<u64, CacheError> {
    cache
        .invalidate_pattern(&format!("patient:{}", pattern))
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_patient_cache_key_format() {
        let id = Uuid::nil();
        let key = patient_cache_key(id);
        assert_eq!(key, format!("patient:{}", id));
    }

}
