use std::sync::Arc;

use base64::{engine::general_purpose::STANDARD, Engine as _};
use wiremock::matchers::{body_string_contains, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use kraken_api_client::auth::StaticCredentials;
use kraken_api_client::spot::rest::private::{
    DepositMethodsRequest, DepositStatusRequest, EarnAllocateRequest,
    EarnAllocationStatusRequest, EarnStrategiesRequest, TransferStatusRequest,
    WalletTransferRequest, WithdrawCancelRequest, WithdrawInfoRequest, WithdrawStatusRequest,
};
use kraken_api_client::spot::rest::SpotRestClient;
use rust_decimal::Decimal;

fn build_client(server: &MockServer) -> SpotRestClient {
    let secret = STANDARD.encode("test_secret");
    let credentials = Arc::new(StaticCredentials::new("test_key", secret));
    SpotRestClient::builder()
        .base_url(server.uri())
        .credentials(credentials)
        .build()
}

#[tokio::test]
async fn test_get_deposit_methods() {
    let server = MockServer::start().await;
    let response = serde_json::json!({
        "error": [],
        "result": [{
            "method": "Bitcoin",
            "limit": false,
            "fee": null,
            "address_setup_fee": null,
            "gen_address": true,
            "minimum": "0.0001"
        }]
    });

    Mock::given(method("POST"))
        .and(path("/0/private/DepositMethods"))
        .and(body_string_contains("asset=XBT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(&server)
        .await;

    let client = build_client(&server);
    let request = DepositMethodsRequest::new("XBT");
    let methods = client.get_deposit_methods(&request).await.unwrap();

    assert_eq!(methods.len(), 1);
    assert_eq!(methods[0].method, "Bitcoin");
    assert!(methods[0].limit.is_none());
}

#[tokio::test]
async fn test_get_withdraw_info() {
    let server = MockServer::start().await;
    let response = serde_json::json!({
        "error": [],
        "result": {
            "method": "Bitcoin",
            "limit": "10.0",
            "fee": "0.0005",
            "amount": "1.0"
        }
    });

    Mock::given(method("POST"))
        .and(path("/0/private/WithdrawInfo"))
        .and(body_string_contains("asset=USDT"))
        .and(body_string_contains("key=main"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(&server)
        .await;

    let client = build_client(&server);
    let request = WithdrawInfoRequest::new("USDT", "main", "1.0".parse().unwrap());
    let info = client.get_withdraw_info(&request).await.unwrap();

    assert_eq!(info.method, "Bitcoin");
    assert!(info.limit.is_some());
}

#[tokio::test]
async fn test_list_earn_strategies() {
    let server = MockServer::start().await;
    let response = serde_json::json!({
        "error": [],
        "result": {
            "items": [{
                "allocation_fee": "0.1",
                "allocation_restriction_info": [],
                "apr_estimate": { "low": "0.01", "high": "0.05" },
                "asset": "XBT",
                "auto_compound": { "type": "enabled", "default": true },
                "can_allocate": true,
                "can_deallocate": true,
                "deallocation_fee": "0.0",
                "id": "STRAT-1",
                "lock_type": { "type": "bonded" },
                "user_cap": "100.0",
                "user_min_allocation": "0.01",
                "yield_source": { "type": "staking" }
            }],
            "next_cursor": null
        }
    });

    Mock::given(method("POST"))
        .and(path("/0/private/Earn/Strategies"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(&server)
        .await;

    let client = build_client(&server);
    let strategies = client
        .list_earn_strategies(Some(&EarnStrategiesRequest::default()))
        .await
        .unwrap();

    assert_eq!(strategies.items.len(), 1);
    assert_eq!(strategies.items[0].asset, "XBT");
}

#[tokio::test]
async fn test_get_deposit_status_list() {
    let server = MockServer::start().await;
    let response = serde_json::json!({
        "error": [],
        "result": [{
            "method": "Bitcoin",
            "aclass": "currency",
            "asset": "XBT",
            "refid": "REF-1",
            "txid": "TX-1",
            "info": "test",
            "amount": "0.5",
            "fee": "0.0001",
            "time": 1700000000,
            "status": "success",
            "status-prop": "onhold",
            "originators": ["addr1"]
        }]
    });

    Mock::given(method("POST"))
        .and(path("/0/private/DepositStatus"))
        .and(body_string_contains("asset=XBT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(&server)
        .await;

    let client = build_client(&server);
    let request = DepositStatusRequest {
        asset: Some("XBT".to_string()),
        ..TransferStatusRequest::default()
    };
    let status = client
        .get_deposit_status(Some(&request))
        .await
        .unwrap();

    assert_eq!(status.entries().len(), 1);
}

#[tokio::test]
async fn test_get_withdraw_status_cursor() {
    let server = MockServer::start().await;
    let response = serde_json::json!({
        "error": [],
        "result": {
            "withdraw": [{
                "method": "Bitcoin",
                "aclass": "currency",
                "asset": "XBT",
                "refid": "REF-2",
                "txid": "TX-2",
                "info": "test",
                "amount": "0.25",
                "fee": "0.0002",
                "time": 1700000001,
                "status": "pending"
            }],
            "cursor": true
        }
    });

    Mock::given(method("POST"))
        .and(path("/0/private/WithdrawStatus"))
        .and(body_string_contains("cursor=true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(&server)
        .await;

    let client = build_client(&server);
    let request = WithdrawStatusRequest {
        cursor: Some(kraken_api_client::spot::rest::private::Cursor::Bool(true)),
        ..TransferStatusRequest::default()
    };
    let status = client
        .get_withdraw_status(Some(&request))
        .await
        .unwrap();

    assert_eq!(status.entries().len(), 1);
    assert!(status.cursor().is_some());
}

#[tokio::test]
async fn test_withdraw_cancel() {
    let server = MockServer::start().await;
    let response = serde_json::json!({
        "error": [],
        "result": true
    });

    Mock::given(method("POST"))
        .and(path("/0/private/WithdrawCancel"))
        .and(body_string_contains("asset=XBT"))
        .and(body_string_contains("refid=REF-3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(&server)
        .await;

    let client = build_client(&server);
    let request = WithdrawCancelRequest::new("XBT", "REF-3");
    let cancelled = client.withdraw_cancel(&request).await.unwrap();

    assert!(cancelled);
}

#[tokio::test]
async fn test_wallet_transfer() {
    let server = MockServer::start().await;
    let response = serde_json::json!({
        "error": [],
        "result": { "refid": "TRANSFER-1" }
    });

    Mock::given(method("POST"))
        .and(path("/0/private/WalletTransfer"))
        .and(body_string_contains("asset=USDT"))
        .and(body_string_contains("from=spot"))
        .and(body_string_contains("to=futures"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(&server)
        .await;

    let client = build_client(&server);
    let request = WalletTransferRequest::new("USDT", "spot", "futures", Decimal::new(10, 0));
    let confirmation = client.wallet_transfer(&request).await.unwrap();

    assert_eq!(confirmation.ref_id, "TRANSFER-1");
}

#[tokio::test]
async fn test_earn_allocate_and_status() {
    let server = MockServer::start().await;
    let allocate_response = serde_json::json!({
        "error": [],
        "result": true
    });
    let status_response = serde_json::json!({
        "error": [],
        "result": { "pending": true }
    });

    Mock::given(method("POST"))
        .and(path("/0/private/Earn/Allocate"))
        .and(body_string_contains("strategy_id=STRAT-2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(allocate_response))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/0/private/Earn/AllocateStatus"))
        .and(body_string_contains("strategy_id=STRAT-2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(status_response))
        .mount(&server)
        .await;

    let client = build_client(&server);
    let allocate_request = EarnAllocateRequest::new(Decimal::new(1, 0), "STRAT-2");
    let accepted = client.earn_allocate(&allocate_request).await.unwrap();
    assert!(accepted);

    let status_request = EarnAllocationStatusRequest::new("STRAT-2");
    let status = client
        .get_earn_allocation_status(&status_request)
        .await
        .unwrap();
    assert!(status.pending);
}
