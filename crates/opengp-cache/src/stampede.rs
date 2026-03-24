use std::time::Duration;

#[derive(Debug, Clone)]
pub struct StampedeGuard {
    pub lock_key: String,
    pub ttl: Duration,
}

impl StampedeGuard {
    pub fn new(cache_key: &str, ttl: Duration) -> Self {
        Self {
            lock_key: format!("{}:lock", cache_key),
            ttl,
        }
    }

    pub fn default_ttl() -> Duration {
        Duration::from_secs(5)
    }

    pub const RETRY_ATTEMPTS: usize = 3;

    pub const RETRY_DELAY: Duration = Duration::from_millis(100);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use tokio::sync::{Barrier, Mutex};

    struct SimulatedCache {
        value: Mutex<Option<u32>>,
        lock_held: Mutex<bool>,
    }

    async fn simulated_get_or_fetch(
        cache: Arc<SimulatedCache>,
        db_queries: Arc<AtomicUsize>,
        barrier: Arc<Barrier>,
    ) -> u32 {
        barrier.wait().await;

        if let Some(cached) = *cache.value.lock().await {
            return cached;
        }

        let acquired_lock = {
            let mut lock = cache.lock_held.lock().await;
            if !*lock {
                *lock = true;
                true
            } else {
                false
            }
        };

        if acquired_lock {
            db_queries.fetch_add(1, Ordering::SeqCst);
            tokio::time::sleep(Duration::from_millis(150)).await;

            {
                let mut value = cache.value.lock().await;
                *value = Some(42);
            }

            {
                let mut lock = cache.lock_held.lock().await;
                *lock = false;
            }

            return 42;
        }

        for _ in 0..StampedeGuard::RETRY_ATTEMPTS {
            tokio::time::sleep(StampedeGuard::RETRY_DELAY).await;
            if let Some(cached) = *cache.value.lock().await {
                return cached;
            }
        }

        db_queries.fetch_add(1, Ordering::SeqCst);
        {
            let mut value = cache.value.lock().await;
            *value = Some(42);
        }
        42
    }

    #[test]
    fn builds_lock_key_with_suffix() {
        let guard = StampedeGuard::new("opengp:patient:123", Duration::from_secs(5));
        assert_eq!(guard.lock_key, "opengp:patient:123:lock");
        assert_eq!(guard.ttl, Duration::from_secs(5));
    }

    #[test]
    fn default_lock_ttl_is_five_seconds() {
        assert_eq!(StampedeGuard::default_ttl(), Duration::from_secs(5));
    }

    #[tokio::test]
    async fn test_cache_stampede_prevention() {
        let cache = Arc::new(SimulatedCache {
            value: Mutex::new(None),
            lock_held: Mutex::new(false),
        });
        let db_queries = Arc::new(AtomicUsize::new(0));
        let barrier = Arc::new(Barrier::new(20));

        let mut handles = Vec::with_capacity(20);
        for _ in 0..20 {
            let cache = cache.clone();
            let db_queries = db_queries.clone();
            let barrier = barrier.clone();

            handles.push(tokio::spawn(async move {
                simulated_get_or_fetch(cache, db_queries, barrier).await
            }));
        }

        for handle in handles {
            assert_eq!(handle.await.unwrap(), 42);
        }

        assert_eq!(db_queries.load(Ordering::SeqCst), 1);
    }
}
