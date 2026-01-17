//! Rate limiting for Kraken API.
//!
//! Kraken has strict rate limits that vary by endpoint type and verification tier.
//! This module provides automatic rate limiting to prevent API bans.
//!
//! ## Rate Limit Categories
//!
//! - **Public endpoints**: Limited by IP address (sliding window)
//! - **Private endpoints**: Limited by API key, varies by verification tier (token bucket)
//! - **Trading endpoints**: Additional penalties for order placement/cancellation
//!
//! ## Example
//!
//! ```rust,ignore
//! use kraken_api_client::spot::rest::SpotRestClient;
//! use kraken_api_client::rate_limit::{RateLimitedClient, RateLimitConfig};
//! use kraken_api_client::types::VerificationTier;
//!
//! // Wrap a client with automatic rate limiting
//! let client = SpotRestClient::new();
//! let rate_limited = RateLimitedClient::new(client, RateLimitConfig {
//!     tier: VerificationTier::Intermediate,
//!     enabled: true,
//! });
//!
//! // All requests are automatically rate limited
//! let time = rate_limited.get_server_time().await?;
//! ```
//!
//! ## Low-Level Rate Limiters
//!
//! You can also use the rate limiters directly for custom logic:
//!
//! ```rust
//! use kraken_api_client::rate_limit::{TtlCache, KeyedRateLimiter, TradingRateLimiter};
//! use std::time::Duration;
//!
//! // Track orders for rate limit penalty calculation
//! let mut order_cache: TtlCache<String, i64> = TtlCache::new(Duration::from_secs(300));
//!
//! // Per-pair rate limiting for order book requests
//! let mut pair_limiter: KeyedRateLimiter<String> = KeyedRateLimiter::new(Duration::from_secs(1), 5);
//!
//! // Trading rate limiter with order lifetime penalties
//! let mut trading_limiter = TradingRateLimiter::new(20, 1.0);
//! ```

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
