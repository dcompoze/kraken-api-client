use std::sync::Arc;

use kraken_api_client::auth::EnvCredentials;
use kraken_api_client::spot::rest::private::{EarnAllocateRequest, EarnStrategiesRequest};
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

    let strategies = client
        .list_earn_strategies(Some(&EarnStrategiesRequest::default()))
        .await?;
    println!("Earn strategies: {}", strategies.items.len());

    if let Some(strategy) = strategies.items.first() {
        let request = EarnAllocateRequest::new(Decimal::new(1, 0), &strategy.id);
        let pending = client.earn_allocate(&request).await?;
        println!("Allocate request accepted: {}", pending);
    }

    Ok(())
}
