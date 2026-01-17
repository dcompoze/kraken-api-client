//! Example: LastAndData wrappers.
//!
//! Run with: cargo run --example types_last_and_data

use kraken_api_client::types::last_and_data::{LastAndData, LastAndDataWithKey};
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wrapped = LastAndData::new("12345", vec![1, 2, 3]);
    let mapped = wrapped.map(|values| values.into_iter().sum::<i32>());
    println!("Last: {}, Sum: {}", mapped.last, mapped.data);

    // Deserialize LastAndDataWithKey from map-like JSON.
    let value = json!({
        "last": "99",
        "XBTUSD": ["1", "2", "3"]
    });
    let decoded: LastAndDataWithKey<Vec<String>> = serde_json::from_value(value)?;
    println!("Decoded last: {}, key: {}", decoded.last, decoded.key);

    Ok(())
}
