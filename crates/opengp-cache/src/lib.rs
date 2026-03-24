//! Cache management for OpenGP
//!
//! Provides Redis-backed caching with connection pooling and circuit breaker pattern.

pub mod appointment_cache;
pub mod circuit;
pub mod error;
pub mod patient_cache;
pub mod pool;
pub mod search_cache;
pub mod service;
pub mod stampede;

pub use appointment_cache::{
    get_appointment_slots, invalidate_all_practice_appointments, invalidate_appointment_slots,
    set_appointment_slots,
};
pub use circuit::{CircuitBreaker, CircuitState};
pub use error::CacheError;
pub use patient_cache::{
    get_patient_by_id, invalidate_patient, invalidate_patient_pattern, set_patient,
    set_patient_default_ttl,
};
pub use pool::RedisPool;
pub use search_cache::{
    get_search_results, get_search_results_by_query, invalidate_all_search, set_search_results,
    set_search_results_by_query, set_search_results_default_ttl,
};
pub use service::{CacheConfig, CacheService, CacheServiceImpl};
pub use stampede::StampedeGuard;
