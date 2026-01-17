//! Example: Authentication helpers and signing.
//!
//! Run with: cargo run --example auth_credentials

use kraken_api_client::auth::{
    Credentials, CredentialsProvider, EnvCredentials, IncreasingNonce, NonceProvider,
    StaticCredentials, sign_request,
};
use kraken_api_client::futures::sign_futures_request;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Static credentials (typically used in tests or explicit config).
    let static_creds = StaticCredentials::new("api_key", "YXBpX3NlY3JldA==");
    println!("Static key: {}", static_creds.get_credentials().api_key);

    // Environment credentials are convenient for local dev.
    if let Some(env_creds) = EnvCredentials::try_from_env() {
        println!(
            "Loaded env credentials: {}",
            env_creds.get_credentials().api_key
        );
    } else {
        println!("Set KRAKEN_API_KEY and KRAKEN_API_SECRET to load env credentials.");
    }

    // Nonce generation for authenticated requests.
    let nonce = IncreasingNonce::new();
    let next_nonce = nonce.next_nonce();
    println!("Next nonce: {}", next_nonce);

    // Spot REST signature example.
    let spot_creds = Credentials::new("api_key", "YXBpX3NlY3JldA==");
    let spot_sig = sign_request(
        &spot_creds,
        "/0/private/Balance",
        1234567890,
        "nonce=1234567890",
    )?;
    println!("Spot signature: {}", spot_sig);

    // Futures REST signature example.
    let futures_sig = sign_futures_request(
        &spot_creds,
        "/api/v3/sendorder",
        1234567890,
        "symbol=PI_XBTUSD&side=buy&orderType=lmt&size=1&limitPrice=10000",
    )?;
    println!("Futures signature: {}", futures_sig);

    Ok(())
}
