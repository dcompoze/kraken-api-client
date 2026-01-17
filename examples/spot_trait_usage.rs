//! Example: Using the KrakenClient trait.
//!
//! Run with: cargo run --example spot_trait_usage

use kraken_api_client::spot::rest::{KrakenClient, SpotRestClient};

async fn print_time<C: KrakenClient>(client: &C) -> Result<(), kraken_api_client::KrakenError> {
    let time = client.get_server_time().await?;
    println!("Server time: {}", time.rfc1123);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SpotRestClient::new();
    print_time(&client).await?;
    Ok(())
}
