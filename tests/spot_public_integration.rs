use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use kraken_api_client::error::KrakenError;
use kraken_api_client::spot::rest::public::{AssetInfoRequest, OhlcRequest, RecentTradesRequest};
use kraken_api_client::spot::rest::SpotRestClient;
use kraken_api_client::types::OhlcInterval;

fn build_public_client(server: &MockServer) -> SpotRestClient {
    SpotRestClient::builder().base_url(server.uri()).build()
}

#[tokio::test]
async fn test_get_server_time() {
    let server = MockServer::start().await;
    let response = serde_json::json!({
        "error": [],
        "result": {
            "unixtime": 1_700_000_000,
            "rfc1123": "Fri, 01 Dec 2023 00:00:00 GMT"
        }
    });

    Mock::given(method("GET"))
        .and(path("/0/public/Time"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(&server)
        .await;

    let client = build_public_client(&server);
    let time = client.get_server_time().await.unwrap();
    assert_eq!(time.unixtime, 1_700_000_000);
    assert_eq!(time.rfc1123, "Fri, 01 Dec 2023 00:00:00 GMT");
}

#[tokio::test]
async fn test_get_assets_with_params() {
    let server = MockServer::start().await;
    let response = serde_json::json!({
        "error": [],
        "result": {
            "XBT": {
                "aclass": "currency",
                "altname": "XBT",
                "decimals": 10,
                "display_decimals": 5
            }
        }
    });

    Mock::given(method("GET"))
        .and(path("/0/public/Assets"))
        .and(query_param("asset", "XBT"))
        .and(query_param("aclass", "currency"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(&server)
        .await;

    let client = build_public_client(&server);
    let request = AssetInfoRequest {
        asset: Some("XBT".to_string()),
        aclass: Some("currency".to_string()),
    };
    let assets = client.get_assets(Some(&request)).await.unwrap();
    assert_eq!(assets.len(), 1);
    assert_eq!(assets["XBT"].altname, "XBT");
}

#[tokio::test]
async fn test_get_ohlc_with_interval_and_since() {
    let server = MockServer::start().await;
    let response = serde_json::json!({
        "error": [],
        "result": {
            "XXBTZUSD": [
                [1_700_000_000, "100.0", "110.0", "90.0", "105.0", "102.0", "1.23", 42]
            ],
            "last": 1_700_000_000
        }
    });

    Mock::given(method("GET"))
        .and(path("/0/public/OHLC"))
        .and(query_param("pair", "XBTUSD"))
        .and(query_param("interval", "60"))
        .and(query_param("since", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(&server)
        .await;

    let client = build_public_client(&server);
    let request = OhlcRequest::new("XBTUSD")
        .interval(OhlcInterval::Hour1)
        .since(1);
    let ohlc = client.get_ohlc(&request).await.unwrap();
    let entries = ohlc.data.get("XXBTZUSD").unwrap();
    assert_eq!(entries[0].count, 42);
}

#[tokio::test]
async fn test_get_recent_trades_parsing() {
    let server = MockServer::start().await;
    let response = serde_json::json!({
        "error": [],
        "result": {
            "XXBTZUSD": [
                ["50000.1", "0.1", 1_700_000_000.1, "b", "l", "", 123]
            ],
            "last": "456"
        }
    });

    Mock::given(method("GET"))
        .and(path("/0/public/Trades"))
        .and(query_param("pair", "XBTUSD"))
        .and(query_param("count", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(&server)
        .await;

    let client = build_public_client(&server);
    let request = RecentTradesRequest::new("XBTUSD").count(1);
    let trades = client.get_recent_trades(&request).await.unwrap();
    let entry = trades.trades.get("XXBTZUSD").unwrap()[0].clone();
    assert_eq!(entry.side, "b");
    assert_eq!(entry.trade_id, 123);
}

#[tokio::test]
async fn test_rate_limit_error_mapping() {
    let server = MockServer::start().await;
    let response = serde_json::json!({
        "error": ["EAPI:Rate limit exceeded"],
        "result": null
    });

    Mock::given(method("GET"))
        .and(path("/0/public/Time"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(&server)
        .await;

    let client = build_public_client(&server);
    let err = client.get_server_time().await.unwrap_err();
    match err {
        KrakenError::RateLimitExceeded { .. } => {}
        other => panic!("unexpected error: {other:?}"),
    }
}
