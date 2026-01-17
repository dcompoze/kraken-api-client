//! Example: Rate limit helpers and wrappers.
//!
//! Run with: cargo run --example rate_limit

use std::time::Duration;

use kraken_api_client::rate_limit::{
    KeyedRateLimiter, OrderTrackingInfo, PerPairTradingLimiter, RateLimitConfig, RateLimitedClient,
    SlidingWindow, TradingRateLimiter, TtlCache,
};
use kraken_api_client::spot::rest::SpotRestClient;
use kraken_api_client::types::VerificationTier;

fn main() {
    // TTL cache for tracking items with expiration.
    let mut cache: TtlCache<String, i64> = TtlCache::new(Duration::from_secs(5));
    let order_key = "order-1".to_string();
    cache.insert(order_key.clone(), 100);
    println!("Cache contains order-1: {}", cache.contains(&order_key));
    if let Some(age) = cache.get_age(&order_key) {
        println!("order-1 age: {:?}", age);
    }

    // Sliding window limiter for a single key.
    let mut window = SlidingWindow::new(Duration::from_secs(1), 2);
    println!("Window allow #1: {:?}", window.try_acquire());
    println!("Window allow #2: {:?}", window.try_acquire());
    println!("Window remaining: {}", window.remaining());

    // Keyed rate limiter across many keys.
    let mut keyed = KeyedRateLimiter::new(Duration::from_secs(1), 3);
    let pair_key = "XBTUSD".to_string();
    let _ = keyed.try_acquire(pair_key.clone());
    println!("Remaining XBTUSD: {}", keyed.remaining(&pair_key));

    // Trading rate limiter with order penalties.
    let mut trading = TradingRateLimiter::new(20, 1.0);
    let info = OrderTrackingInfo::with_client_id("XBTUSD", "client-1");
    trading.track_order("order-123", info);
    println!("Tracked orders: {}", trading.tracked_orders());
    let cancel_result = trading.try_cancel_order("order-123");
    println!("Cancel result: {:?}", cancel_result);
    trading.order_cancelled("order-123");
    trading.order_filled("order-123");

    // Per-pair trading limiter.
    let mut per_pair = PerPairTradingLimiter::new(20, 1.0);
    let limiter = per_pair.limiter_for("ETHUSD");
    println!("ETHUSD capacity: {}", limiter.available_capacity());

    // Rate-limited wrapper around a REST client.
    let inner = SpotRestClient::new();
    let config = RateLimitConfig {
        tier: VerificationTier::Intermediate,
        enabled: true,
    };
    let mut wrapped = RateLimitedClient::new(inner, config);
    wrapped.set_enabled(true);
    println!("Rate limiting enabled: {}", wrapped.config().enabled);
}
