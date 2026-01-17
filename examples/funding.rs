use std::sync::Arc;

use kraken_api_client::auth::EnvCredentials;
use kraken_api_client::spot::rest::private::{
    DepositAddressesRequest, DepositMethodsRequest, WithdrawMethodsRequest,
};
use kraken_api_client::spot::rest::SpotRestClient;

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

    let methods = client
        .get_deposit_methods(&DepositMethodsRequest::new("XBT"))
        .await?;
    println!("Deposit methods: {}", methods.len());

    let addresses = client
        .get_deposit_addresses(&DepositAddressesRequest::new("XBT", "Bitcoin"))
        .await?;
    println!("Deposit addresses: {}", addresses.len());

    let withdraw_methods = client
        .get_withdraw_methods(Some(&WithdrawMethodsRequest::default()))
        .await?;
    println!("Withdraw methods: {}", withdraw_methods.len());

    Ok(())
}
