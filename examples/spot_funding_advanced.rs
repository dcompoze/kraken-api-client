//! Example: Spot funding endpoints (advanced).
//!
//! Run with: cargo run --example spot_funding_advanced

use std::env;
use std::str::FromStr;
use std::sync::Arc;

use kraken_api_client::auth::EnvCredentials;
use kraken_api_client::spot::rest::private::{
    DepositStatusRequest, WithdrawAddressesRequest, WithdrawCancelRequest, WithdrawInfoRequest,
    WithdrawRequest, WithdrawStatusRequest, WalletTransferRequest,
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

    println!("=== Deposit Status ===");
    let deposits = client
        .get_deposit_status(Some(&DepositStatusRequest {
            asset: Some("XBT".into()),
            ..Default::default()
        }))
        .await?;
    println!("Deposits: {}", deposits.entries().len());

    println!("\n=== Withdraw Addresses ===");
    let withdraw_addresses = client
        .get_withdraw_addresses(Some(&WithdrawAddressesRequest {
            asset: Some("XBT".into()),
            ..Default::default()
        }))
        .await?;
    println!("Withdraw addresses: {}", withdraw_addresses.len());

    if let (Ok(key), Ok(amount)) = (
        env::var("KRAKEN_WITHDRAW_KEY"),
        env::var("KRAKEN_WITHDRAW_AMOUNT"),
    ) {
        println!("\n=== Withdraw Info ===");
        let amount = Decimal::from_str(&amount)?;
        let info_request = WithdrawInfoRequest::new("XBT", key.clone(), amount);
        let info = client.get_withdraw_info(&info_request).await?;
        println!("Withdraw fee: {}", info.fee);

        if env::var("KRAKEN_DO_WITHDRAW").is_ok() {
            println!("\n=== Withdraw Funds (Dangerous) ===");
            let withdraw_request = WithdrawRequest::new("XBT", key, amount)
                .max_fee(info.fee);
            let confirm = client.withdraw_funds(&withdraw_request).await?;
            println!("Withdraw reference: {}", confirm.ref_id);
        } else {
            println!("Set KRAKEN_DO_WITHDRAW=1 to execute a real withdrawal.");
        }
    } else {
        println!("Set KRAKEN_WITHDRAW_KEY and KRAKEN_WITHDRAW_AMOUNT to run withdraw info.");
    }

    println!("\n=== Withdraw Status ===");
    let withdrawals = client
        .get_withdraw_status(Some(&WithdrawStatusRequest {
            asset: Some("XBT".into()),
            ..Default::default()
        }))
        .await?;
    println!("Withdrawals: {}", withdrawals.entries().len());

    if let (Ok(asset), Ok(ref_id)) = (
        env::var("KRAKEN_WITHDRAW_CANCEL_ASSET"),
        env::var("KRAKEN_WITHDRAW_CANCEL_REFID"),
    ) {
        let cancel_request = WithdrawCancelRequest::new(asset, ref_id);
        let cancelled = client.withdraw_cancel(&cancel_request).await?;
        println!("Withdrawal cancelled: {}", cancelled);
    } else {
        println!("Set KRAKEN_WITHDRAW_CANCEL_ASSET and KRAKEN_WITHDRAW_CANCEL_REFID to cancel.");
    }

    if env::var("KRAKEN_WALLET_TRANSFER").is_ok() {
        let asset = env::var("KRAKEN_WALLET_TRANSFER_ASSET").unwrap_or_else(|_| "XBT".to_string());
        let from = env::var("KRAKEN_WALLET_TRANSFER_FROM").unwrap_or_else(|_| "Spot".to_string());
        let to = env::var("KRAKEN_WALLET_TRANSFER_TO").unwrap_or_else(|_| "Futures".to_string());
        let amount = env::var("KRAKEN_WALLET_TRANSFER_AMOUNT").unwrap_or_else(|_| "0.001".to_string());
        let amount = Decimal::from_str(&amount)?;

        println!("\n=== Wallet Transfer (Dangerous) ===");
        let transfer = WalletTransferRequest::new(asset, from, to, amount);
        let result = client.wallet_transfer(&transfer).await?;
        println!("Transfer reference: {}", result.ref_id);
    } else {
        println!("Set KRAKEN_WALLET_TRANSFER=1 to execute a wallet transfer.");
    }

    Ok(())
}
