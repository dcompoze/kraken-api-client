//! Trading-specific rate limiting with order lifetime penalties.
//!
//! Kraken applies different rate limit penalties for order cancellation
//! based on how long the order has been open. Orders cancelled quickly
//! incur higher penalties.
//!
//! # Penalty Schedule
//!
//! | Order Age | Cancel Penalty |
//! |-----------|----------------|
//! | < 5s      | 8 points       |
//! | 5-10s     | 6 points       |
//! | 10-15s    | 5 points       |
//! | 15-45s    | 4 points       |
//! | 45-90s    | 2 points       |
//! | > 90s     | 0 points       |

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::rate_limit::limits::trading;
use crate::rate_limit::TtlCache;

/// Information tracked for each order.
#[derive(Debug, Clone)]
pub struct OrderTrackingInfo {
    /// When the order was placed.
    pub created_at: Instant,
    /// Trading pair for the order.
    pub pair: String,
    /// Client order ID if provided.
    pub client_order_id: Option<String>,
}

impl OrderTrackingInfo {
    /// Create new order tracking info.
    pub fn new(pair: impl Into<String>) -> Self {
        Self {
            created_at: Instant::now(),
            pair: pair.into(),
            client_order_id: None,
        }
    }

    /// Create new order tracking info with a client order ID.
    pub fn with_client_id(pair: impl Into<String>, client_order_id: impl Into<String>) -> Self {
        Self {
            created_at: Instant::now(),
            pair: pair.into(),
            client_order_id: Some(client_order_id.into()),
        }
    }

    /// Get the age of this order.
    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }
}

/// Trading rate limiter with order lifetime penalty tracking.
///
/// Tracks orders and calculates appropriate rate limit penalties
/// when they are cancelled based on their age.
#[derive(Debug)]
pub struct TradingRateLimiter {
    /// Order tracking cache (orders expire after 5 minutes)
    orders: TtlCache<String, OrderTrackingInfo>,
    /// Current rate limit counter (scaled 100x for precision)
    counter: i64,
    /// Maximum counter value (scaled 100x)
    max_counter: i64,
    /// Decay rate per second (scaled 100x)
    decay_rate: i64,
    /// Last time the counter was updated
    last_update: Instant,
}

impl TradingRateLimiter {
    /// Create a new trading rate limiter.
    ///
    /// # Arguments
    ///
    /// * `max_counter` - Maximum counter value
    /// * `decay_rate_per_sec` - How much the counter decays per second
    pub fn new(max_counter: u32, decay_rate_per_sec: f64) -> Self {
        Self {
            orders: TtlCache::new(Duration::from_secs(300)), // 5 minute TTL
            counter: 0,
            max_counter: (max_counter as i64) * 100,
            decay_rate: (decay_rate_per_sec * 100.0) as i64,
            last_update: Instant::now(),
        }
    }

    /// Update the counter based on time decay.
    fn update_counter(&mut self) {
        let elapsed = self.last_update.elapsed();
        let elapsed_secs = elapsed.as_secs_f64();
        let decay = (elapsed_secs * self.decay_rate as f64) as i64;
        self.counter = (self.counter - decay).max(0);
        self.last_update = Instant::now();
    }

    /// Try to acquire capacity for a new order.
    ///
    /// Returns `Ok(())` if allowed, `Err(wait_time)` if rate limited.
    pub fn try_place_order(&mut self, order_id: &str, info: OrderTrackingInfo) -> Result<(), Duration> {
        self.update_counter();

        // Adding an order costs 1 point (100 in scaled units)
        let cost = 100;

        if self.counter + cost <= self.max_counter {
            self.counter += cost;
            self.orders.insert(order_id.to_string(), info);
            Ok(())
        } else {
            // Calculate wait time
            let excess = self.counter + cost - self.max_counter;
            let wait_secs = excess as f64 / self.decay_rate as f64;
            Err(Duration::from_secs_f64(wait_secs))
        }
    }

    /// Track an order that was placed (without rate limit check).
    ///
    /// Use this when the order was already placed successfully.
    pub fn track_order(&mut self, order_id: impl Into<String>, info: OrderTrackingInfo) {
        self.orders.insert(order_id.into(), info);
    }

    /// Calculate the penalty for cancelling an order.
    ///
    /// Returns the penalty in points based on the order's age.
    pub fn cancel_penalty(age: Duration) -> u32 {
        let secs = age.as_secs();

        if secs < 5 {
            trading::CANCEL_PENALTY_UNDER_5S
        } else if secs < 10 {
            trading::CANCEL_PENALTY_5_TO_10S
        } else if secs < 15 {
            trading::CANCEL_PENALTY_10_TO_15S
        } else if secs < 45 {
            trading::CANCEL_PENALTY_15_TO_45S
        } else if secs < 90 {
            trading::CANCEL_PENALTY_45_TO_90S
        } else {
            trading::CANCEL_PENALTY_OVER_90S
        }
    }

    /// Try to cancel an order with rate limit penalty.
    ///
    /// Returns `Ok(penalty)` if allowed (with the penalty that was applied),
    /// or `Err(wait_time)` if rate limited.
    pub fn try_cancel_order(&mut self, order_id: &str) -> Result<u32, Duration> {
        self.update_counter();

        // Get the order age and calculate penalty
        let penalty = if let Some((_, age)) = self.orders.remove_with_age(&order_id.to_string()) {
            Self::cancel_penalty(age)
        } else {
            // Order not tracked, assume worst case
            trading::CANCEL_PENALTY_UNDER_5S
        };

        let cost = (penalty as i64) * 100;

        if self.counter + cost <= self.max_counter {
            self.counter += cost;
            Ok(penalty)
        } else {
            // Calculate wait time
            let excess = self.counter + cost - self.max_counter;
            let wait_secs = excess as f64 / self.decay_rate as f64;
            Err(Duration::from_secs_f64(wait_secs))
        }
    }

    /// Notify the limiter that an order was cancelled (without rate limit check).
    ///
    /// Use this when the cancellation was already processed.
    pub fn order_cancelled(&mut self, order_id: &str) {
        self.orders.remove(&order_id.to_string());
    }

    /// Notify the limiter that an order was filled.
    ///
    /// Filled orders don't incur cancellation penalties.
    pub fn order_filled(&mut self, order_id: &str) {
        self.orders.remove(&order_id.to_string());
    }

    /// Get the current counter value (unscaled).
    pub fn current_counter(&self) -> f64 {
        let elapsed = self.last_update.elapsed();
        let elapsed_secs = elapsed.as_secs_f64();
        let decay = elapsed_secs * self.decay_rate as f64;
        let counter = (self.counter as f64 - decay).max(0.0);
        counter / 100.0
    }

    /// Get the available capacity (unscaled).
    pub fn available_capacity(&self) -> f64 {
        (self.max_counter as f64 / 100.0) - self.current_counter()
    }

    /// Check if placing an order would be allowed.
    pub fn would_allow_place(&self) -> bool {
        let current = (self.current_counter() * 100.0) as i64;
        current + 100 <= self.max_counter
    }

    /// Get the number of tracked orders.
    pub fn tracked_orders(&self) -> usize {
        self.orders.active_count()
    }

    /// Clean up expired order tracking entries.
    pub fn cleanup(&mut self) {
        self.orders.cleanup();
    }
}

impl Default for TradingRateLimiter {
    fn default() -> Self {
        // Default: Pro tier limits
        Self::new(20, 1.0)
    }
}

/// Per-pair trading rate limiter.
///
/// Maintains separate rate limits for each trading pair.
#[derive(Debug, Default)]
pub struct PerPairTradingLimiter {
    limiters: HashMap<String, TradingRateLimiter>,
    max_counter: u32,
    decay_rate: f64,
}

impl PerPairTradingLimiter {
    /// Create a new per-pair trading limiter.
    pub fn new(max_counter: u32, decay_rate: f64) -> Self {
        Self {
            limiters: HashMap::new(),
            max_counter,
            decay_rate,
        }
    }

    /// Get or create a limiter for a specific pair.
    pub fn limiter_for(&mut self, pair: &str) -> &mut TradingRateLimiter {
        self.limiters
            .entry(pair.to_string())
            .or_insert_with(|| TradingRateLimiter::new(self.max_counter, self.decay_rate))
    }

    /// Clean up all limiters.
    pub fn cleanup(&mut self) {
        for limiter in self.limiters.values_mut() {
            limiter.cleanup();
        }
    }

    /// Get the number of pairs being tracked.
    pub fn tracked_pairs(&self) -> usize {
        self.limiters.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_cancel_penalty_calculation() {
        assert_eq!(TradingRateLimiter::cancel_penalty(Duration::from_secs(2)), 8);
        assert_eq!(TradingRateLimiter::cancel_penalty(Duration::from_secs(5)), 6);
        assert_eq!(TradingRateLimiter::cancel_penalty(Duration::from_secs(12)), 5);
        assert_eq!(TradingRateLimiter::cancel_penalty(Duration::from_secs(30)), 4);
        assert_eq!(TradingRateLimiter::cancel_penalty(Duration::from_secs(60)), 2);
        assert_eq!(TradingRateLimiter::cancel_penalty(Duration::from_secs(100)), 0);
    }

    #[test]
    fn test_place_order_tracking() {
        let mut limiter = TradingRateLimiter::new(20, 1.0);

        let info = OrderTrackingInfo::new("BTC/USD");
        assert!(limiter.try_place_order("order1", info).is_ok());
        assert_eq!(limiter.tracked_orders(), 1);
    }

    #[test]
    fn test_cancel_penalty_applied() {
        let mut limiter = TradingRateLimiter::new(20, 1.0);

        let info = OrderTrackingInfo::new("BTC/USD");
        limiter.try_place_order("order1", info).ok();

        // Cancel immediately (should get max penalty)
        let result = limiter.try_cancel_order("order1");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 8); // Under 5s penalty
    }

    #[test]
    fn test_decay_over_time() {
        let mut limiter = TradingRateLimiter::new(20, 10.0); // High decay rate for testing

        // Fill up the counter
        for i in 0..20 {
            let info = OrderTrackingInfo::new("BTC/USD");
            limiter.try_place_order(&format!("order{}", i), info).ok();
        }

        let initial = limiter.current_counter();
        thread::sleep(Duration::from_millis(200));
        let after = limiter.current_counter();

        assert!(after < initial);
    }

    #[test]
    fn test_order_info_age() {
        let info = OrderTrackingInfo::new("BTC/USD");
        thread::sleep(Duration::from_millis(50));
        let age = info.age();
        assert!(age >= Duration::from_millis(50));
    }
}
