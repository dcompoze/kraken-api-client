use std::sync::Arc;

use base64::{Engine as _, engine::general_purpose::STANDARD};
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use kraken_api_client::auth::NonceProvider;
use kraken_api_client::auth::{Credentials, StaticCredentials};
use kraken_api_client::error::KrakenError;
use kraken_api_client::futures::rest::AccountLogRequest;
use kraken_api_client::futures::rest::FuturesRestClient;
use kraken_api_client::futures::sign_futures_request;

fn build_public_client(server: &MockServer) -> FuturesRestClient {
    FuturesRestClient::builder()
        .base_url(server.uri())
        .build()
        .unwrap()
}

struct FixedNonce(u64);

impl NonceProvider for FixedNonce {
    fn next_nonce(&self) -> u64 {
        self.0
    }
}

#[tokio::test]
async fn test_get_tickers() {
    let server = MockServer::start().await;
    let response = serde_json::json!({
        "result": "success",
        "tickers": [
            { "symbol": "PI_XBTUSD", "last": "50000.0" }
        ],
        "serverTime": "1700000000"
    });

    Mock::given(method("GET"))
        .and(path("/api/v3/tickers"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(&server)
        .await;

    let client = build_public_client(&server);
    let tickers = client.get_tickers().await.unwrap();
    assert_eq!(tickers.len(), 1);
    assert_eq!(tickers[0].symbol, "PI_XBTUSD");
}

#[tokio::test]
async fn test_get_orderbook_with_symbol_param() {
    let server = MockServer::start().await;
    let response = serde_json::json!({
        "result": "success",
        "orderBook": {
            "symbol": "PI_XBTUSD",
            "bids": [{ "price": "50000.0", "size": "1" }],
            "asks": [{ "price": "50010.0", "size": "2" }]
        }
    });

    Mock::given(method("GET"))
        .and(path("/api/v3/orderbook"))
        .and(query_param("symbol", "PI_XBTUSD"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(&server)
        .await;

    let client = build_public_client(&server);
    let book = client.get_orderbook("PI_XBTUSD").await.unwrap();
    assert_eq!(book.symbol, "PI_XBTUSD");
    assert_eq!(book.bids.len(), 1);
}

#[tokio::test]
async fn test_private_get_accounts_signs_request() {
    let server = MockServer::start().await;
    let secret = STANDARD.encode("test_secret");
    let credentials = Arc::new(StaticCredentials::new("test_key", &secret));
    let nonce = 12345;
    let signature = sign_futures_request(
        &Credentials::new("test_key", secret),
        "/api/v3/accounts",
        nonce,
        "",
    )
    .unwrap();
    let response = serde_json::json!({
        "result": "success",
        "accounts": {
            "cash": { "type": "cash", "currency": "USD" }
        }
    });

    Mock::given(method("GET"))
        .and(path("/api/v3/accounts"))
        .and(header("APIKey", "test_key"))
        .and(header("Nonce", nonce.to_string()))
        .and(header("Authent", signature))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(&server)
        .await;

    let client = FuturesRestClient::builder()
        .base_url(server.uri())
        .credentials(credentials)
        .nonce_provider(Arc::new(FixedNonce(nonce)))
        .danger_allow_insecure_transport()
        .build()
        .unwrap();

    let accounts = client.get_accounts().await.unwrap();
    assert_eq!(accounts.result, "success");
    assert!(accounts.accounts.contains_key("cash"));
}

#[tokio::test]
async fn test_get_fee_schedules() {
    let server = MockServer::start().await;
    let response = serde_json::json!({
        "result": "success",
        "feeSchedules": [
            {
                "uid": "5b755fea-c5b0-4307-a66e-b392cd5bd931",
                "name": "KF USD Multi-Collateral Fees",
                "tiers": [
                    { "makerFee": 0.02, "takerFee": 0.05, "usdVolume": 0.0 },
                    { "makerFee": 0.015, "takerFee": 0.04, "usdVolume": 100000.0 }
                ]
            }
        ],
        "serverTime": "2024-01-15T10:00:00Z"
    });

    Mock::given(method("GET"))
        .and(path("/api/v3/feeschedules"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(&server)
        .await;

    let client = build_public_client(&server);
    let schedules = client.get_fee_schedules().await.unwrap();
    assert_eq!(schedules.len(), 1);
    assert_eq!(schedules[0].name, "KF USD Multi-Collateral Fees");
    assert_eq!(schedules[0].tiers.len(), 2);
}

#[tokio::test]
async fn test_get_historical_funding_rates() {
    let server = MockServer::start().await;
    let response = serde_json::json!({
        "result": "success",
        "rates": [
            {
                "timestamp": "2024-01-15T08:00:00.000Z",
                "fundingRate": 1.0e-9,
                "relativeFundingRate": 7.5e-7
            }
        ],
        "serverTime": "2024-01-15T10:00:00Z"
    });

    Mock::given(method("GET"))
        .and(path("/api/v4/historicalfundingrates"))
        .and(query_param("symbol", "PI_XBTUSD"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(&server)
        .await;

    let client = build_public_client(&server);
    let rates = client
        .get_historical_funding_rates("PI_XBTUSD")
        .await
        .unwrap();
    assert_eq!(rates.len(), 1);
    assert_eq!(rates[0].timestamp, "2024-01-15T08:00:00.000Z");
}

#[tokio::test]
async fn test_get_account_log_signs_request() {
    let server = MockServer::start().await;
    let secret = STANDARD.encode("test_secret");
    let credentials = Arc::new(StaticCredentials::new("test_key", &secret));
    let nonce = 42;
    let signature = sign_futures_request(
        &Credentials::new("test_key", secret),
        "/api/history/v2/account-log",
        nonce,
        "count=2",
    )
    .unwrap();
    let response = serde_json::json!({
        "accountUid": "f7d5571c-6d10-4cf1-944a-048d25682ed0",
        "logs": [
            {
                "id": 10,
                "date": "2023-04-04T16:10:46.260Z",
                "info": "admin transfer",
                "asset": "eth",
                "margin_account": "ETH",
                "old_balance": 0.0,
                "new_balance": 2.6868286287
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/api/history/v2/account-log"))
        .and(query_param("count", "2"))
        .and(header("APIKey", "test_key"))
        .and(header("Nonce", nonce.to_string()))
        .and(header("Authent", signature))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(&server)
        .await;

    let client = FuturesRestClient::builder()
        .base_url(server.uri())
        .credentials(credentials)
        .nonce_provider(Arc::new(FixedNonce(nonce)))
        .danger_allow_insecure_transport()
        .build()
        .unwrap();

    let request = AccountLogRequest {
        count: Some(2),
        ..Default::default()
    };
    let log = client.get_account_log(Some(&request)).await.unwrap();
    assert_eq!(log.logs.len(), 1);
    assert_eq!(log.logs[0].info, "admin transfer");
}

#[tokio::test]
async fn test_set_leverage_preference_signs_put_request() {
    let server = MockServer::start().await;
    let secret = STANDARD.encode("test_secret");
    let credentials = Arc::new(StaticCredentials::new("test_key", &secret));
    let nonce = 7;
    let signature = sign_futures_request(
        &Credentials::new("test_key", secret),
        "/api/v3/leveragepreferences",
        nonce,
        "symbol=PF_XBTUSD&maxLeverage=10",
    )
    .unwrap();
    let response = serde_json::json!({
        "result": "success",
        "serverTime": "2024-01-15T10:00:00Z"
    });

    Mock::given(method("PUT"))
        .and(path("/api/v3/leveragepreferences"))
        .and(query_param("symbol", "PF_XBTUSD"))
        .and(query_param("maxLeverage", "10"))
        .and(header("APIKey", "test_key"))
        .and(header("Nonce", nonce.to_string()))
        .and(header("Authent", signature))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(&server)
        .await;

    let client = FuturesRestClient::builder()
        .base_url(server.uri())
        .credentials(credentials)
        .nonce_provider(Arc::new(FixedNonce(nonce)))
        .danger_allow_insecure_transport()
        .build()
        .unwrap();

    let result = client
        .set_leverage_preference("PF_XBTUSD", Some(rust_decimal::Decimal::from(10)))
        .await
        .unwrap();
    assert_eq!(result.result, "success");
}

#[tokio::test]
async fn test_futures_error_response_maps_to_api_error() {
    let server = MockServer::start().await;
    let response = serde_json::json!({
        "result": "error",
        "error": "Not authorized"
    });

    Mock::given(method("GET"))
        .and(path("/api/v3/tickers"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(&server)
        .await;

    let client = build_public_client(&server);
    let err = client.get_tickers().await.unwrap_err();
    match err {
        KrakenError::Api(api_error) => {
            assert_eq!(api_error.code, "EFutures");
        }
        other => panic!("unexpected error: {other:?}"),
    }
}
