//! Search result cache integration
//!
//! Provides high-level functions for caching and retrieving patient search results.
//! Uses SHA256 hashing of search queries as cache keys for consistency.

use opengp_domain::domain::patient::Patient;
use sha2::{Digest, Sha256};

use crate::error::CacheError;
use crate::service::CacheServiceImpl;

/// Compute SHA256 hash of a search query
fn hash_query(query: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(query.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Build a cache key for search results
fn search_cache_key(query_hash: &str) -> String {
    format!("search:{}", query_hash)
}

/// Get cached search results by query hash
///
/// # Arguments
/// * `cache` - Cache service instance
/// * `query_hash` - SHA256 hash of the original search query
///
/// # Returns
/// * `Ok(Some(Vec<Patient>))` if search results are found in cache
/// * `Ok(None)` if search results are not in cache
/// * `Err(CacheError)` if cache operation fails
pub async fn get_search_results(
    cache: &CacheServiceImpl,
    query_hash: &str,
) -> Result<Option<Vec<Patient>>, CacheError> {
    let key = search_cache_key(query_hash);
    cache.get::<Vec<Patient>>(&key).await
}

/// Get cached search results by query string (hashes the query)
///
/// # Arguments
/// * `cache` - Cache service instance
/// * `query` - Original search query string
///
/// # Returns
/// * `Ok(Some(Vec<Patient>))` if search results are found in cache
/// * `Ok(None)` if search results are not in cache
/// * `Err(CacheError)` if cache operation fails
pub async fn get_search_results_by_query(
    cache: &CacheServiceImpl,
    query: &str,
) -> Result<Option<Vec<Patient>>, CacheError> {
    let query_hash = hash_query(query);
    get_search_results(cache, &query_hash).await
}

/// Set cached search results
///
/// # Arguments
/// * `cache` - Cache service instance
/// * `query_hash` - SHA256 hash of the original search query
/// * `results` - Search results to cache
/// * `ttl` - Time to live in seconds
///
/// # Returns
/// * `Ok(())` if successful
/// * `Err(CacheError)` if cache operation fails
pub async fn set_search_results(
    cache: &CacheServiceImpl,
    query_hash: &str,
    results: &[Patient],
    ttl: u64,
) -> Result<(), CacheError> {
    let key = search_cache_key(query_hash);
    cache.set(&key, &results.to_vec(), Some(ttl)).await
}

/// Set cached search results with default TTL
///
/// # Arguments
/// * `cache` - Cache service instance
/// * `query_hash` - SHA256 hash of the original search query
/// * `results` - Search results to cache
///
/// # Returns
/// * `Ok(())` if successful
/// * `Err(CacheError)` if cache operation fails
pub async fn set_search_results_default_ttl(
    cache: &CacheServiceImpl,
    query_hash: &str,
    results: &[Patient],
) -> Result<(), CacheError> {
    set_search_results(cache, query_hash, results, cache.search_ttl_secs()).await
}

/// Set cached search results by query string (hashes the query)
///
/// # Arguments
/// * `cache` - Cache service instance
/// * `query` - Original search query string
/// * `results` - Search results to cache
/// * `ttl` - Time to live in seconds
///
/// # Returns
/// * `Ok(())` if successful
/// * `Err(CacheError)` if cache operation fails
pub async fn set_search_results_by_query(
    cache: &CacheServiceImpl,
    query: &str,
    results: &[Patient],
    ttl: u64,
) -> Result<(), CacheError> {
    let query_hash = hash_query(query);
    set_search_results(cache, &query_hash, results, ttl).await
}

/// Invalidate all search cache entries
///
/// # Arguments
/// * `cache` - Cache service instance
///
/// # Returns
/// * `Ok(count)` - Number of keys deleted
/// * `Err(CacheError)` if cache operation fails
pub async fn invalidate_all_search(cache: &CacheServiceImpl) -> Result<u64, CacheError> {
    cache.invalidate_pattern("search:*").await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_query_deterministic() {
        let query = "John Doe";
        let hash1 = hash_query(query);
        let hash2 = hash_query(query);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_query_different_inputs() {
        let hash1 = hash_query("John Doe");
        let hash2 = hash_query("Jane Doe");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_search_cache_key_format() {
        let hash = "abc123";
        let key = search_cache_key(hash);
        assert_eq!(key, "search:abc123");
    }
}
