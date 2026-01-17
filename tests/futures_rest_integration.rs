use std::sync::Arc;

use base64::{Engine as _, engine::general_purpose::STANDARD};
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use kraken_api_client::auth::NonceProvider;
use kraken_api_client::auth::{Credentials, StaticCredentials};
use kraken_api_client::error::KrakenError;
use kraken_api_client::futures::rest::FuturesRestClient;
use kraken_api_client::futures::sign_futures_request;

fn build_public_client(server: &MockServer) -> FuturesRestClient {
    FuturesRestClient::builder().base_url(server.uri()).build()
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
        .build();

    let accounts = client.get_accounts().await.unwrap();
    assert_eq!(accounts.result, "success");
    assert!(accounts.accounts.contains_key("cash"));
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
