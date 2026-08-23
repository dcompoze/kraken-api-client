//! Rate limiting for the Kraken API.
//!
//! Kraken limits public endpoints by IP address (sliding window), private endpoints by API key and verification tier (token bucket), and applies extra penalties for order placement and cancellation.
//! [`RateLimitedClient`] wraps a client with automatic rate limiting, and the individual limiters can be used directly for custom logic.

mod client;
mod keyed;
mod trading;
mod ttl_cache;

pub use client::RateLimitedClient;
pub use keyed::{KeyedRateLimiter, SlidingWindow};
pub use trading::{OrderTrackingInfo, PerPairTradingLimiter, TradingRateLimiter};
pub use ttl_cache::TtlCache;

use crate::types::VerificationTier;

/// Rate limiter configuration.
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Verification tier (affects rate limits).
    pub tier: VerificationTier,
    /// Whether to enable rate limiting.
    pub enabled: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            tier: VerificationTier::Starter,
            enabled: true,
        }
    }
}

/// Rate limit constants by verification tier.
pub mod limits {
    /// Starter tier limits.
    pub mod starter {
        /// Maximum API counter value.
        pub const MAX_COUNTER: u32 = 15;
        /// Counter decay rate per second.
        pub const DECAY_RATE: f64 = 0.33;
    }

    /// Intermediate tier limits.
    pub mod intermediate {
        /// Maximum API counter value.
        pub const MAX_COUNTER: u32 = 20;
        /// Counter decay rate per second.
        pub const DECAY_RATE: f64 = 0.5;
    }

    /// Pro tier limits.
    pub mod pro {
        /// Maximum API counter value.
        pub const MAX_COUNTER: u32 = 20;
        /// Counter decay rate per second.
        pub const DECAY_RATE: f64 = 1.0;
    }

    /// Trading rate limit constants.
    pub mod trading {
        /// Maximum orders per second.
        pub const MAX_ORDERS_PER_SECOND: u32 = 60;
        /// Penalty for orders under 5 seconds old when cancelled.
        pub const CANCEL_PENALTY_UNDER_5S: u32 = 8;
        /// Penalty for orders 5-10 seconds old when cancelled.
        pub const CANCEL_PENALTY_5_TO_10S: u32 = 6;
        /// Penalty for orders 10-15 seconds old when cancelled.
        pub const CANCEL_PENALTY_10_TO_15S: u32 = 5;
        /// Penalty for orders 15-45 seconds old when cancelled.
        pub const CANCEL_PENALTY_15_TO_45S: u32 = 4;
        /// Penalty for orders 45-90 seconds old when cancelled.
        pub const CANCEL_PENALTY_45_TO_90S: u32 = 2;
        /// Penalty for orders over 90 seconds old when cancelled.
        pub const CANCEL_PENALTY_OVER_90S: u32 = 0;
    }
}
