use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::service::CacheCircuitBreakerConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

struct CircuitInner {
    state: CircuitState,
    failure_count: u32,
    opened_at: Option<Instant>,
    probe_in_flight: bool,
}

pub struct CircuitBreaker {
    inner: Arc<Mutex<CircuitInner>>,
    failure_threshold: u32,
    open_duration: Duration,
}

impl CircuitBreaker {
    pub fn new() -> Self {
        Self::with_config(5, Duration::from_secs(30))
    }

    pub fn from_config(config: &CacheCircuitBreakerConfig) -> Self {
        Self::with_config(
            config.failure_threshold,
            Duration::from_secs(config.open_duration_secs),
        )
    }

    pub fn with_config(failure_threshold: u32, open_duration: Duration) -> Self {
        Self {
            inner: Arc::new(Mutex::new(CircuitInner {
                state: CircuitState::Closed,
                failure_count: 0,
                opened_at: None,
                probe_in_flight: false,
            })),
            failure_threshold,
            open_duration,
        }
    }

    pub fn state(&self) -> CircuitState {
        self.inner.lock().unwrap().state
    }

    pub fn allow_request(&self) -> bool {
        let mut inner = self.inner.lock().unwrap();
        match inner.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if inner
                    .opened_at
                    .is_some_and(|opened| opened.elapsed() >= self.open_duration)
                {
                    inner.state = CircuitState::HalfOpen;
                    inner.probe_in_flight = true;
                    true
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => {
                if inner.probe_in_flight {
                    false
                } else {
                    inner.probe_in_flight = true;
                    true
                }
            }
        }
    }

    pub fn record_success(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.failure_count = 0;
        inner.state = CircuitState::Closed;
        inner.opened_at = None;
        inner.probe_in_flight = false;
    }

    pub fn record_failure(&self) {
        let mut inner = self.inner.lock().unwrap();

        match inner.state {
            CircuitState::Closed => {
                inner.failure_count += 1;
                if inner.failure_count >= self.failure_threshold {
                    inner.state = CircuitState::Open;
                    inner.opened_at = Some(Instant::now());
                    inner.probe_in_flight = false;
                }
            }
            CircuitState::HalfOpen => {
                inner.state = CircuitState::Open;
                inner.opened_at = Some(Instant::now());
                inner.probe_in_flight = false;
            }
            CircuitState::Open => {
                inner.opened_at = Some(Instant::now());
                inner.probe_in_flight = false;
            }
        }
    }

    pub fn attempt_reset(&self) {
        let mut inner = self.inner.lock().unwrap();
        if inner.state == CircuitState::Open {
            inner.state = CircuitState::HalfOpen;
            inner.probe_in_flight = false;
        }
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opens_after_five_consecutive_failures() {
        let breaker = CircuitBreaker::new();

        for _ in 0..4 {
            breaker.record_failure();
            assert_eq!(breaker.state(), CircuitState::Closed);
        }

        breaker.record_failure();
        assert_eq!(breaker.state(), CircuitState::Open);
    }

    #[test]
    fn half_open_probe_success_closes_circuit() {
        let breaker = CircuitBreaker::with_config(5, Duration::from_millis(1));
        for _ in 0..5 {
            breaker.record_failure();
        }

        std::thread::sleep(Duration::from_millis(5));
        assert!(breaker.allow_request());
        assert_eq!(breaker.state(), CircuitState::HalfOpen);

        breaker.record_success();
        assert_eq!(breaker.state(), CircuitState::Closed);
    }

    #[test]
    fn half_open_probe_failure_reopens_circuit() {
        let breaker = CircuitBreaker::with_config(5, Duration::from_millis(1));
        for _ in 0..5 {
            breaker.record_failure();
        }

        std::thread::sleep(Duration::from_millis(5));
        assert!(breaker.allow_request());
        breaker.record_failure();
        assert_eq!(breaker.state(), CircuitState::Open);
        assert!(!breaker.allow_request());
    }
}
