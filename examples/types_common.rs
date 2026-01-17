//! Example: Common types and enums.
//!
//! Run with: cargo run --example types_common

use kraken_api_client::types::{
    AssetClass, BuySell, LedgerType, OhlcInterval, OrderFlag, OrderStatus, OrderType, SelfTradePrevent,
    TimeInForce, TriggerType, VerificationTier,
};

fn main() {
    let side = BuySell::Buy;
    let order_type = OrderType::Limit;
    let status = OrderStatus::Open;
    println!("Side: {}, Type: {}, Status: {}", side, order_type, status);

    let tif = TimeInForce::GTC;
    let oflag = OrderFlag::Post;
    let trigger = TriggerType::Last;
    let stp = SelfTradePrevent::CancelOldest;
    let asset_class = AssetClass::Currency;
    let ledger = LedgerType::Trade;
    println!(
        "Other enums: {:?}, {:?}, {:?}, {:?}, {:?}, {:?}",
        tif, oflag, trigger, stp, asset_class, ledger
    );

    let interval = OhlcInterval::Hour1;
    let interval_secs: u32 = interval.into();
    let roundtrip = OhlcInterval::try_from(interval_secs).unwrap();
    println!("Interval: {:?} -> {} -> {:?}", interval, interval_secs, roundtrip);

    let tier = VerificationTier::Intermediate;
    let (max_counter, decay_rate) = tier.rate_limit_params();
    println!("Tier: {:?}, max={}, decay_rate={}", tier, max_counter, decay_rate);
}
