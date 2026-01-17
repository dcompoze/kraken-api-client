use std::sync::Arc;

use kraken_api_client::auth::EnvCredentials;
use kraken_api_client::futures::rest::FuturesRestClient;
use kraken_api_client::spot::rest::SpotRestClient;

fn live_tests_enabled() -> bool {
    std::env::var("KRAKEN_LIVE_TESTS").ok().as_deref() == Some("1")
}

#[tokio::test]
#[ignore]
async fn live_spot_private_smoke() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenv::dotenv();
    if !live_tests_enabled() {
        return Ok(());
    }

    let credentials = match EnvCredentials::try_from_env() {
        Some(creds) => creds,
        None => return Ok(()),
    };
    let client = SpotRestClient::builder()
        .credentials(Arc::new(credentials))
        .build();

    let _balances = client.get_account_balance().await?;
    let token = client.get_websocket_token().await?;
    assert!(!token.token.is_empty());

    Ok(())
}

#[tokio::test]
#[ignore]
async fn live_futures_private_smoke() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenv::dotenv();
    if !live_tests_enabled() {
        return Ok(());
    }

    let credentials = match EnvCredentials::try_from_env() {
        Some(creds) => creds,
        None => return Ok(()),
    };
    let client = FuturesRestClient::builder()
        .credentials(Arc::new(credentials))
        .build();

    let accounts = client.get_accounts().await?;
    assert_eq!(accounts.result, "success");

    Ok(())
}
