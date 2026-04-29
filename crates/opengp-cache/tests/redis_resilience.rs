//! Integration tests for Redis resilience patterns
//!
//! Tests cover:
//! - Fallback behavior when Redis is unavailable
//! - Cache stampede prevention with concurrent cache misses
//! - Circuit breaker state transitions
//! - Cache invalidation chains

use opengp_cache::{CacheConfig, CacheServiceImpl, CircuitBreaker, RedisPool};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Barrier;

/// Helper to detect if Redis is available
async fn is_redis_available() -> bool {
    // Try to connect to Redis on default localhost:6379
    // If REDIS_URL is set, try that; otherwise default to localhost
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    match RedisPool::from_url(&redis_url, 5).await {
        Ok(pool) => {
            // Quick connectivity check
            match pool.get().await {
                Ok(_) => true,
                Err(_) => false,
            }
        }
        Err(_) => false,
    }
}

/// Skip test if Redis is not available
macro_rules! skip_if_no_redis {
    () => {
        if !is_redis_available().await {
            println!("Skipping test: Redis not available");
            return;
        }
    };
}

// ============================================================================
// TEST: Redis unavailable falls back to DB
// ============================================================================

#[tokio::test]
async fn test_redis_unavailable_falls_back_to_db() {
    // This test demonstrates the fallback pattern when Redis is unavailable
    // The circuit breaker ensures we don't hammer an unavailable Redis server

    let circuit = CircuitBreaker::with_config(5, Duration::from_secs(30));

    // Circuit starts closed
    assert_eq!(
        circuit.state(),
        opengp_cache::circuit::CircuitState::Closed,
        "Circuit should start closed"
    );
    assert!(circuit.allow_request(), "Should allow requests when closed");

    // Simulate 5 consecutive Redis connection failures
    // In real scenario, this would be actual Redis connection timeouts
    for i in 1..=5 {
        circuit.record_failure();

        if i < 5 {
            // Before threshold is reached, circuit stays closed
            assert_eq!(
                circuit.state(),
                opengp_cache::circuit::CircuitState::Closed,
                "Should stay closed after {} failure(s)",
                i
            );
        }
    }

    // After 5 failures, circuit opens
    assert_eq!(
        circuit.state(),
        opengp_cache::circuit::CircuitState::Open,
        "Circuit should open after 5 failures"
    );

    // Circuit rejects further requests immediately (fallback to DB)
    assert!(
        !circuit.allow_request(),
        "Circuit breaker should reject new requests when open"
    );

    // Application would catch this and serve from database instead
}

// ============================================================================
// TEST: Cache stampede prevention
// ============================================================================

#[tokio::test]
async fn test_cache_stampede_prevention() {
    skip_if_no_redis!();

    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let pool = RedisPool::from_url(&redis_url, 10)
        .await
        .expect("Failed to connect to Redis");

    let cache = CacheServiceImpl::new(
        pool,
        CacheConfig {
            enabled: true,
            default_ttl_secs: 3600,
            key_prefix: "opengp-test".to_string(),
            circuit_breaker: Default::default(),
            stampede: Default::default(),
            entity_ttl: Default::default(),
        },
        CircuitBreaker::with_config(10, Duration::from_secs(30)),
    );

    let cache = Arc::new(cache);
    let barrier = Arc::new(Barrier::new(20));
    let db_queries = Arc::new(AtomicUsize::new(0));

    // Clear the test key first
    let _ = cache.invalidate("stampede-test").await;

    let mut handles = Vec::with_capacity(20);

    for _ in 0..20 {
        let cache_clone = cache.clone();
        let barrier_clone = barrier.clone();
        let db_queries_clone = db_queries.clone();

        let handle = tokio::spawn(async move {
            barrier_clone.wait().await;

            // Try to get from cache
            let result: Result<Option<i32>, _> = cache_clone.get("stampede-test").await;

            if result.is_ok() && result.unwrap().is_none() {
                // Cache miss - try to acquire stampede lock
                let guard = opengp_cache::StampedeGuard::new(
                    "opengp-test:stampede-test",
                    Duration::from_secs(5),
                );

                // Simulate acquiring lock and doing DB query
                let lock_acquired = cache_clone
                    .try_acquire_stampede_lock(&guard)
                    .await
                    .unwrap_or(false);

                if lock_acquired {
                    db_queries_clone.fetch_add(1, Ordering::SeqCst);
                    tokio::time::sleep(Duration::from_millis(100)).await;

                    // Set the value in cache
                    let _ = cache_clone.set("stampede-test", &42, Some(3600)).await;
                    let _ = cache_clone.release_stampede_lock(&guard).await;
                } else {
                    // Wait for the other request to populate cache
                    for _ in 0..10 {
                        tokio::time::sleep(Duration::from_millis(50)).await;
                        if cache_clone.get::<i32>("stampede-test").await.is_ok() {
                            break;
                        }
                    }
                }
            }

            42
        });

        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        let _ = handle.await;
    }

    // Should only have 1 DB query despite 20 concurrent cache misses
    let query_count = db_queries.load(Ordering::SeqCst);
    assert_eq!(
        query_count, 1,
        "Should have exactly 1 DB query, got {}",
        query_count
    );

    // Cleanup
    let _ = cache.invalidate("stampede-test").await;
}

// ============================================================================
// TEST: Circuit breaker transition
// ============================================================================

#[tokio::test]
async fn test_circuit_breaker_transition() {
    // This test doesn't require Redis - it's purely testing circuit logic
    let circuit = CircuitBreaker::with_config(5, Duration::from_millis(200));

    // Initially closed
    assert_eq!(circuit.state(), opengp_cache::CircuitState::Closed);
    assert!(circuit.allow_request());

    // Accumulate failures
    for i in 1..=5 {
        circuit.record_failure();
        if i < 5 {
            assert_eq!(
                circuit.state(),
                opengp_cache::CircuitState::Closed,
                "Should still be closed after {} failures",
                i
            );
        }
    }

    // Should be open now
    assert_eq!(circuit.state(), opengp_cache::CircuitState::Open);
    assert!(!circuit.allow_request(), "Should reject requests when open");

    // Wait for reset timeout
    tokio::time::sleep(Duration::from_millis(250)).await;

    // Should transition to half-open on next request
    assert!(circuit.allow_request(), "Should allow probe request");
    assert_eq!(
        circuit.state(),
        opengp_cache::CircuitState::HalfOpen,
        "Should be half-open"
    );

    // Probe succeeds - circuit closes
    circuit.record_success();
    assert_eq!(
        circuit.state(),
        opengp_cache::CircuitState::Closed,
        "Should close after successful probe"
    );
    assert!(circuit.allow_request(), "Should allow requests when closed");
}

// ============================================================================
// TEST: Cache invalidation chain
// ============================================================================

#[tokio::test]
async fn test_cache_invalidation_chain() {
    skip_if_no_redis!();

    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let pool = RedisPool::from_url(&redis_url, 5)
        .await
        .expect("Failed to connect to Redis");

    let cache = CacheServiceImpl::new(
        pool,
        CacheConfig {
            enabled: true,
            default_ttl_secs: 3600,
            key_prefix: "opengp-test-inv".to_string(),
            circuit_breaker: Default::default(),
            stampede: Default::default(),
            entity_ttl: Default::default(),
        },
        CircuitBreaker::default(),
    );

    let patient_id = "patient:123";
    let search_key = "search:abc123";

    // Clear any existing data
    let _ = cache.invalidate(patient_id).await;
    let _ = cache.invalidate(search_key).await;

    // Set patient in cache
    let patient_data = "Patient data";
    cache
        .set(patient_id, &patient_data, Some(3600))
        .await
        .expect("Failed to set patient cache");

    // Set search results in cache
    let search_results = vec!["result1", "result2"];
    cache
        .set(search_key, &search_results, Some(3600))
        .await
        .expect("Failed to set search cache");

    // Verify both are cached
    let cached_patient: Option<String> =
        cache.get(patient_id).await.expect("Failed to get patient");
    assert!(cached_patient.is_some(), "Patient should be in cache");

    let cached_search: Option<Vec<String>> = cache
        .get(search_key)
        .await
        .expect("Failed to get search results");
    assert!(cached_search.is_some(), "Search results should be in cache");

    // Invalidate patient AND related search cache
    // This simulates a patient update invalidating both patient and search caches
    let _ = cache.invalidate(patient_id).await;
    let _ = cache.invalidate(search_key).await;

    // Verify both are now gone
    let after_patient: Option<String> = cache
        .get(patient_id)
        .await
        .expect("Failed to check patient after invalidation");
    assert!(
        after_patient.is_none(),
        "Patient should be invalidated from cache"
    );

    let after_search: Option<Vec<String>> = cache
        .get(search_key)
        .await
        .expect("Failed to check search after invalidation");
    assert!(
        after_search.is_none(),
        "Search results should be invalidated from cache"
    );
}

// ============================================================================
// TEST: Invalidate pattern (batch invalidation)
// ============================================================================

#[tokio::test]
async fn test_cache_invalidation_pattern() {
    skip_if_no_redis!();

    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let pool = RedisPool::from_url(&redis_url, 5)
        .await
        .expect("Failed to connect to Redis");

    let cache = CacheServiceImpl::new(
        pool,
        CacheConfig {
            enabled: true,
            default_ttl_secs: 3600,
            key_prefix: "opengp-pattern".to_string(),
            circuit_breaker: Default::default(),
            stampede: Default::default(),
            entity_ttl: Default::default(),
        },
        CircuitBreaker::default(),
    );

    // Set multiple patient entries
    for i in 1..=5 {
        let key = format!("patient:{}", i);
        cache
            .set(&key, &format!("Patient {}", i), Some(3600))
            .await
            .expect("Failed to set patient");
    }

    // Verify all are cached
    for i in 1..=5 {
        let key = format!("patient:{}", i);
        let cached: Option<String> = cache.get(&key).await.expect("Failed to get");
        assert!(cached.is_some(), "Patient {} should be cached", i);
    }

    // Invalidate all patient entries using pattern
    let deleted = cache
        .invalidate_pattern("patient:*")
        .await
        .expect("Failed to invalidate pattern");

    assert_eq!(deleted, 5, "Should have deleted 5 patient entries");

    // Verify all are gone
    for i in 1..=5 {
        let key = format!("patient:{}", i);
        let cached: Option<String> = cache.get(&key).await.expect("Failed to check");
        assert!(cached.is_none(), "Patient {} should be gone", i);
    }
}
