//! Example: Using a boxed client without trait objects.
//!
//! Run with: cargo run --example spot_trait_object

use kraken_api_client::spot::rest::SpotRestClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Box::new(SpotRestClient::new());
    let status = client.get_system_status().await?;
    println!("System status: {}", status.status);
    Ok(())
}
