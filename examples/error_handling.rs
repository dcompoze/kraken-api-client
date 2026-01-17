//! Example: Working with KrakenError and ApiError.
//!
//! Run with: cargo run --example error_handling

use kraken_api_client::error::{error_codes, ApiError};
use kraken_api_client::KrakenError;

fn main() {
    let api_error = ApiError::new("EAPI", "Rate limit exceeded");
    println!("API error: {}", api_error);
    println!("Full code: {}", api_error.full_code());
    println!("Is rate limit: {}", api_error.is_rate_limit());

    let err = KrakenError::Api(api_error.clone());
    match err {
        KrakenError::Api(inner) => {
            if inner.full_code() == error_codes::RATE_LIMIT_EXCEEDED {
                println!("Matched known rate limit error");
            }
        }
        _ => {
            println!("Unexpected error type");
        }
    }
}
