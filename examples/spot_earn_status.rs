//! Example: Spot Earn endpoints.
//!
//! Run with: cargo run --example spot_earn_status

use std::env;
use std::str::FromStr;
use std::sync::Arc;

use kraken_api_client::auth::EnvCredentials;
use kraken_api_client::spot::rest::private::{
    EarnAllocateRequest, EarnAllocationStatusRequest, EarnAllocationsRequest, EarnStrategiesRequest,
};
use kraken_api_client::spot::rest::SpotRestClient;
use rust_decimal::Decimal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let credentials = match EnvCredentials::try_from_env() {
        Some(creds) => Arc::new(creds),
        None => {
            println!("Set KRAKEN_API_KEY and KRAKEN_API_SECRET to run this example.");
            return Ok(());
        }
    };

    let client = SpotRestClient::builder().credentials(credentials).build();

    println!("=== Earn Strategies ===");
    let strategies = client
        .list_earn_strategies(Some(&EarnStrategiesRequest::default()))
        .await?;
    println!("Strategies: {}", strategies.items.len());

    println!("\n=== Earn Allocations ===");
    let allocations = client
        .list_earn_allocations(Some(&EarnAllocationsRequest::default()))
        .await?;
    println!("Allocations: {}", allocations.items.len());

    if let Some(strategy) = strategies.items.first() {
        let amount = env::var("KRAKEN_EARN_AMOUNT").unwrap_or_else(|_| "1".to_string());
        let amount = Decimal::from_str(&amount)?;

        if env::var("KRAKEN_EARN_ALLOCATE").is_ok() {
            let request = EarnAllocateRequest::new(amount, &strategy.id);
            let accepted = client.earn_allocate(&request).await?;
            println!("Allocate accepted: {}", accepted);
        } else {
            println!("Set KRAKEN_EARN_ALLOCATE=1 to allocate funds.");
        }

        let status_request = EarnAllocationStatusRequest::new(&strategy.id);
        let status = client.get_earn_allocation_status(&status_request).await?;
        println!("Allocation pending: {}", status.pending);

        if env::var("KRAKEN_EARN_DEALLOCATE").is_ok() {
            let request = EarnAllocateRequest::new(amount, &strategy.id);
            let accepted = client.earn_deallocate(&request).await?;
            println!("Deallocate accepted: {}", accepted);
        } else {
            println!("Set KRAKEN_EARN_DEALLOCATE=1 to deallocate funds.");
        }

        let dealloc_status = client.get_earn_deallocation_status(&status_request).await?;
        println!("Deallocation pending: {}", dealloc_status.pending);
    }

    Ok(())
}
