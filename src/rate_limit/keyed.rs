//! Per-key rate limiting.
//!
//! This module provides rate limiting that can be applied on a per-key basis,
//! such as per trading pair for order book requests.
//!
//! # Example
//!
//! ```rust
//! use std::time::Duration;
//! use kraken_api_client::rate_limit::KeyedRateLimiter;
//!
//! let mut limiter = KeyedRateLimiter::new(
//!     Duration::from_secs(1),  // Window size
//!     5,                        // Max requests per window
//! );
//!
//! // Check if we can make a request for a specific key
//! assert!(limiter.try_acquire("BTC/USD").is_ok());
//! ```

use std::collections::HashMap;
use std::hash::Hash;
use std::time::{Duration, Instant};

/// Per-key rate limiter using a sliding window algorithm.
///
/// Each key (e.g., trading pair) has its own rate limit tracking.
/// Useful for endpoints like order book that have per-pair limits.
#[derive(Debug)]
pub struct KeyedRateLimiter<K> {
    /// Rate limits per key
    limiters: HashMap<K, SlidingWindow>,
    /// Window duration
    window: Duration,
    /// Maximum requests per window
    max_requests: u32,
}

impl<K> KeyedRateLimiter<K>
where
    K: Hash + Eq + Clone,
{
    /// Create a new per-key rate limiter.
    ///
    /// # Arguments
    ///
    /// * `window` - The sliding window duration
    /// * `max_requests` - Maximum number of requests allowed per window
    pub fn new(window: Duration, max_requests: u32) -> Self {
        Self {
            limiters: HashMap::new(),
            window,
            max_requests,
        }
    }

    /// Try to acquire a permit for the given key.
    ///
    /// Returns `Ok(())` if the request is allowed, or `Err(wait_time)` if
    /// the rate limit has been exceeded and you need to wait.
    pub fn try_acquire(&mut self, key: K) -> Result<(), Duration> {
        let limiter = self
            .limiters
            .entry(key)
            .or_insert_with(|| SlidingWindow::new(self.window, self.max_requests));

        limiter.try_acquire()
    }

    /// Check if a request for the given key would be allowed without consuming a permit.
    pub fn would_allow(&self, key: &K) -> bool {
        self.limiters
            .get(key)
            .is_none_or(|limiter| limiter.would_allow())
    }

    /// Get the remaining permits for a key.
    pub fn remaining(&self, key: &K) -> u32 {
        self.limiters
            .get(key)
            .map_or(self.max_requests, |limiter| limiter.remaining())
    }

    /// Get the time until the next permit is available for a key.
    pub fn time_until_available(&self, key: &K) -> Option<Duration> {
        self.limiters
            .get(key)
            .and_then(|limiter| limiter.time_until_available())
    }

    /// Remove all rate limit tracking for a specific key.
    pub fn remove(&mut self, key: &K) {
        self.limiters.remove(key);
    }

    /// Clean up limiters that haven't been used recently.
    ///
    /// Removes limiters where all requests have expired from the window.
    pub fn cleanup(&mut self) {
        self.limiters.retain(|_, limiter| !limiter.is_empty());
    }

    /// Get the number of keys being tracked.
    pub fn tracked_keys(&self) -> usize {
        self.limiters.len()
    }

    /// Clear all rate limit tracking.
    pub fn clear(&mut self) {
        self.limiters.clear();
    }
}

impl<K> Default for KeyedRateLimiter<K>
where
    K: Hash + Eq + Clone,
{
    fn default() -> Self {
        // Default: 1 request per second per key
        Self::new(Duration::from_secs(1), 1)
    }
}

/// A sliding window rate limiter.
///
/// Tracks request timestamps within a sliding window and enforces a maximum
/// number of requests within that window.
#[derive(Debug)]
pub struct SlidingWindow {
    /// Request timestamps
    requests: Vec<Instant>,
    /// Window duration
    window: Duration,
    /// Maximum requests per window
    max_requests: u32,
}

impl SlidingWindow {
    /// Create a new sliding window rate limiter.
    pub fn new(window: Duration, max_requests: u32) -> Self {
        Self {
            requests: Vec::with_capacity(max_requests as usize),
            window,
            max_requests,
        }
    }

    /// Try to acquire a permit.
    ///
    /// Returns `Ok(())` if allowed, `Err(wait_time)` if rate limited.
    pub fn try_acquire(&mut self) -> Result<(), Duration> {
        self.cleanup_old();

        if (self.requests.len() as u32) < self.max_requests {
            self.requests.push(Instant::now());
            Ok(())
        } else {
            // Find when the oldest request will expire.
            let wait_time = self
                .requests
                .first()
                .map(|oldest| self.window.saturating_sub(oldest.elapsed()))
                .unwrap_or_default();
            Err(wait_time)
        }
    }

    /// Check if a request would be allowed without consuming a permit.
    pub fn would_allow(&self) -> bool {
        let count = self
            .requests
            .iter()
            .filter(|ts| ts.elapsed() < self.window)
            .count();
        (count as u32) < self.max_requests
    }

    /// Get the number of remaining permits.
    pub fn remaining(&self) -> u32 {
        let count = self
            .requests
            .iter()
            .filter(|ts| ts.elapsed() < self.window)
            .count() as u32;
        self.max_requests.saturating_sub(count)
    }

    /// Get the time until the next permit is available.
    ///
    /// Returns `None` if a permit is available now.
    pub fn time_until_available(&self) -> Option<Duration> {
        self.cleanup_check();

        let count = self
            .requests
            .iter()
            .filter(|ts| ts.elapsed() < self.window)
            .count();

        if (count as u32) < self.max_requests {
            None
        } else {
            // Find the oldest request still in the window
            self.requests
                .iter()
                .find(|ts| ts.elapsed() < self.window)
                .map(|oldest| self.window.saturating_sub(oldest.elapsed()))
        }
    }

    /// Check if the window has no active requests.
    pub fn is_empty(&self) -> bool {
        self.requests.iter().all(|ts| ts.elapsed() >= self.window)
    }

    /// Remove requests that are outside the window.
    fn cleanup_old(&mut self) {
        let window = self.window;
        self.requests.retain(|ts| ts.elapsed() < window);
    }

    /// Internal cleanup check (immutable).
    fn cleanup_check(&self) -> usize {
        self.requests
            .iter()
            .filter(|ts| ts.elapsed() < self.window)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_sliding_window_allows_within_limit() {
        let mut limiter = SlidingWindow::new(Duration::from_secs(1), 3);

        assert!(limiter.try_acquire().is_ok());
        assert!(limiter.try_acquire().is_ok());
        assert!(limiter.try_acquire().is_ok());
        assert!(limiter.try_acquire().is_err());
    }

    #[test]
    fn test_sliding_window_resets_after_window() {
        let mut limiter = SlidingWindow::new(Duration::from_millis(50), 2);

        assert!(limiter.try_acquire().is_ok());
        assert!(limiter.try_acquire().is_ok());
        assert!(limiter.try_acquire().is_err());

        thread::sleep(Duration::from_millis(60));

        assert!(limiter.try_acquire().is_ok());
    }

    #[test]
    fn test_remaining() {
        let mut limiter = SlidingWindow::new(Duration::from_secs(1), 3);

        assert_eq!(limiter.remaining(), 3);
        limiter.try_acquire().ok();
        assert_eq!(limiter.remaining(), 2);
        limiter.try_acquire().ok();
        assert_eq!(limiter.remaining(), 1);
    }

    #[test]
    fn test_keyed_limiter() {
        let mut limiter: KeyedRateLimiter<String> =
            KeyedRateLimiter::new(Duration::from_secs(1), 2);

        // Different keys have independent limits
        assert!(limiter.try_acquire("BTC/USD".to_string()).is_ok());
        assert!(limiter.try_acquire("BTC/USD".to_string()).is_ok());
        assert!(limiter.try_acquire("BTC/USD".to_string()).is_err());

        // ETH/USD has its own limit
        assert!(limiter.try_acquire("ETH/USD".to_string()).is_ok());
        assert!(limiter.try_acquire("ETH/USD".to_string()).is_ok());
        assert!(limiter.try_acquire("ETH/USD".to_string()).is_err());
    }

    #[test]
    fn test_keyed_limiter_cleanup() {
        let mut limiter: KeyedRateLimiter<String> =
            KeyedRateLimiter::new(Duration::from_millis(50), 1);

        limiter.try_acquire("key1".to_string()).ok();
        limiter.try_acquire("key2".to_string()).ok();
        assert_eq!(limiter.tracked_keys(), 2);

        thread::sleep(Duration::from_millis(60));
        limiter.cleanup();

        assert_eq!(limiter.tracked_keys(), 0);
    }
}
