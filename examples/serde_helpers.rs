//! Example: serde helper utilities.
//!
//! Run with: cargo run --example serde_helpers

use std::collections::BTreeSet;

use kraken_api_client::types::serde_helpers::{
    comma_separated, default_on_error, display_fromstr, empty_string_as_none, maybe_decimal,
    optional_comma_separated,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Payload {
    #[serde(with = "comma_separated")]
    symbols: BTreeSet<String>,
    #[serde(with = "display_fromstr")]
    size: Decimal,
    #[serde(default, deserialize_with = "default_on_error::deserialize")]
    maybe_number: Option<u32>,
    #[serde(default, deserialize_with = "maybe_decimal::deserialize")]
    maybe_price: Option<Decimal>,
    #[serde(default, deserialize_with = "empty_string_as_none::deserialize")]
    note: Option<String>,
    #[serde(default, with = "optional_comma_separated")]
    tags: Option<BTreeSet<String>>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut symbols = BTreeSet::new();
    symbols.insert("XBTUSD".to_string());
    symbols.insert("ETHUSD".to_string());

    let mut tags = BTreeSet::new();
    tags.insert("maker".to_string());
    tags.insert("fast".to_string());

    let payload = Payload {
        symbols,
        size: Decimal::new(125, 2),
        maybe_number: Some(42),
        maybe_price: Some(Decimal::new(50000, 0)),
        note: Some("ok".to_string()),
        tags: Some(tags),
    };

    let json = serde_json::to_string(&payload)?;
    println!("Serialized: {}", json);

    let decoded: Payload = serde_json::from_str(
        r#"{
            "symbols": "XBTUSD,ETHUSD",
            "size": "1.25",
            "maybe_number": "not-a-number",
            "maybe_price": "",
            "note": "",
            "tags": "maker,fast"
        }"#,
    )?;

    println!("Decoded: {:?}", decoded);
    Ok(())
}
