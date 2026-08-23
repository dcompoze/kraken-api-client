//! Integration tests for spot private trading, query, export, and subaccount endpoints.

use std::str::FromStr;
use std::sync::Arc;

use base64::{Engine as _, engine::general_purpose::STANDARD};
use rust_decimal::Decimal;
use wiremock::matchers::{body_string_contains, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use kraken_api_client::auth::StaticCredentials;
use kraken_api_client::spot::rest::SpotRestClient;
use kraken_api_client::spot::rest::private::{
    AccountTransferRequest, AddExportRequest, AddOrderBatchRequest, AmendOrderRequest, BatchOrder,
    CancelAllOrdersAfterRequest, CancelOrderBatchRequest, CreateSubaccountRequest,
    EditOrderRequest, EditOrderStatus, ExportStatusRequest, OrderAmendsRequest,
    QueryLedgersRequest, QueryTradesRequest, RemoveExportRequest, RemoveExportType, ReportType,
    RetrieveExportRequest,
};
use kraken_api_client::types::{BuySell, OrderType};

fn build_client(server: &MockServer) -> SpotRestClient {
    let secret = STANDARD.encode("test_secret");
    let credentials = Arc::new(StaticCredentials::new("test_key", secret));
    SpotRestClient::builder()
        .base_url(server.uri())
        .credentials(credentials)
        .build()
}

#[tokio::test]
async fn test_edit_order() {
    let server = MockServer::start().await;
    let response = serde_json::json!({
        "error": [],
        "result": {
            "status": "ok",
            "txid": "OFVXHJ-KPQ3B-VS7ELA",
            "originaltxid": "OHYO67-6LP66-HMQ437",
            "volume": "0.00030000",
            "price": "19500.0",
            "orders_cancelled": 1,
            "descr": {
                "order": "buy 0.00030000 XBTUSDT @ limit 19500.0"
            }
        }
    });

    Mock::given(method("POST"))
        .and(path("/0/private/EditOrder"))
        .and(body_string_contains("txid=OHYO67-6LP66-HMQ437"))
        .and(body_string_contains("pair=XBTUSDT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(&server)
        .await;

    let client = build_client(&server);
    let request = EditOrderRequest::new(
        "OHYO67-6LP66-HMQ437",
        "XBTUSDT",
        Decimal::from_str("0.0003").unwrap(),
    )
    .price(Decimal::from_str("19500.0").unwrap());
    let result = client.edit_order(&request).await.unwrap();

    assert_eq!(result.status, EditOrderStatus::Ok);
    assert_eq!(result.txid.as_deref(), Some("OFVXHJ-KPQ3B-VS7ELA"));
    assert_eq!(result.orders_cancelled, Some(1));
}

#[tokio::test]
async fn test_amend_order() {
    let server = MockServer::start().await;
    let response = serde_json::json!({
        "error": [],
        "result": {
            "amend_id": "TJSMEH-AA67V-YUSQ6O"
        }
    });

    Mock::given(method("POST"))
        .and(path("/0/private/AmendOrder"))
        .and(body_string_contains("\"txid\":\"OHYO67-6LP66-HMQ437\""))
        .and(body_string_contains("\"limit_price\":\"19500.0\""))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(&server)
        .await;

    let client = build_client(&server);
    let request = AmendOrderRequest::by_txid("OHYO67-6LP66-HMQ437").limit_price("19500.0");
    let result = client.amend_order(&request).await.unwrap();

    assert_eq!(result.amend_id, "TJSMEH-AA67V-YUSQ6O");
}

#[tokio::test]
async fn test_add_order_batch() {
    let server = MockServer::start().await;
    let response = serde_json::json!({
        "error": [],
        "result": {
            "orders": [
                {
                    "txid": "O5TLGX-DKKTU-WKRAZ5",
                    "descr": { "order": "buy 0.5 XBTUSD @ limit 28800.0" }
                },
                {
                    "txid": "OBGFYP-XVQNL-P4GMWF",
                    "descr": { "order": "sell 0.5 XBTUSD @ limit 32100.0" }
                }
            ]
        }
    });

    Mock::given(method("POST"))
        .and(path("/0/private/AddOrderBatch"))
        .and(body_string_contains("\"pair\":\"XBTUSD\""))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(&server)
        .await;

    let client = build_client(&server);
    let orders = vec![
        BatchOrder::new(
            BuySell::Buy,
            OrderType::Limit,
            Decimal::from_str("0.5").unwrap(),
        )
        .price(Decimal::from_str("28800.0").unwrap()),
        BatchOrder::new(
            BuySell::Sell,
            OrderType::Limit,
            Decimal::from_str("0.5").unwrap(),
        )
        .price(Decimal::from_str("32100.0").unwrap()),
    ];
    let request = AddOrderBatchRequest::new("XBTUSD", orders);
    let result = client.add_order_batch(&request).await.unwrap();

    assert_eq!(result.orders.len(), 2);
    assert_eq!(result.orders[0].txid.as_deref(), Some("O5TLGX-DKKTU-WKRAZ5"));
}

#[tokio::test]
async fn test_cancel_order_batch() {
    let server = MockServer::start().await;
    let response = serde_json::json!({
        "error": [],
        "result": { "count": 2 }
    });

    Mock::given(method("POST"))
        .and(path("/0/private/CancelOrderBatch"))
        .and(body_string_contains("OG5V2Y-RYKVL-DT3V3B"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(&server)
        .await;

    let client = build_client(&server);
    let request = CancelOrderBatchRequest::from_txids(vec![
        "OG5V2Y-RYKVL-DT3V3B".to_string(),
        "OP5V2Y-RYKVL-DT3V3B".to_string(),
    ]);
    let result = client.cancel_order_batch(&request).await.unwrap();

    assert_eq!(result.count, 2);
}

#[tokio::test]
async fn test_cancel_all_orders_after() {
    let server = MockServer::start().await;
    let response = serde_json::json!({
        "error": [],
        "result": {
            "currentTime": "2023-03-24T17:41:56Z",
            "triggerTime": "2023-03-24T17:42:56Z"
        }
    });

    Mock::given(method("POST"))
        .and(path("/0/private/CancelAllOrdersAfter"))
        .and(body_string_contains("timeout=60"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(&server)
        .await;

    let client = build_client(&server);
    let request = CancelAllOrdersAfterRequest::new(60);
    let result = client.cancel_all_orders_after(&request).await.unwrap();

    assert_eq!(result.trigger_time, "2023-03-24T17:42:56Z");
}

#[tokio::test]
async fn test_query_trades() {
    let server = MockServer::start().await;
    let response = serde_json::json!({
        "error": [],
        "result": {
            "THVRQM-33VKH-UCI7BS": {
                "ordertxid": "OQCLML-BW3P3-BUCMWZ",
                "postxid": "TKH2SE-M7IF5-CFI7LT",
                "pair": "XXBTZUSD",
                "time": 1688667796.8802,
                "type": "buy",
                "ordertype": "limit",
                "price": "30010.00000",
                "cost": "600.20000",
                "fee": "0.00000",
                "vol": "0.02000000",
                "margin": "0.00000",
                "misc": ""
            }
        }
    });

    Mock::given(method("POST"))
        .and(path("/0/private/QueryTrades"))
        .and(body_string_contains("txid=THVRQM-33VKH-UCI7BS"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(&server)
        .await;

    let client = build_client(&server);
    let request = QueryTradesRequest::new("THVRQM-33VKH-UCI7BS");
    let trades = client.query_trades(&request).await.unwrap();

    assert_eq!(trades.len(), 1);
    assert!(trades.contains_key("THVRQM-33VKH-UCI7BS"));
}

#[tokio::test]
async fn test_query_ledgers() {
    let server = MockServer::start().await;
    let response = serde_json::json!({
        "error": [],
        "result": {
            "L4UESK-KG3EQ-UFO4T5": {
                "refid": "TJKLXF-PGMUI-4NTLXU",
                "time": 1688464484.1787,
                "type": "trade",
                "subtype": "",
                "aclass": "currency",
                "asset": "ZGBP",
                "amount": "-24.5000",
                "fee": "0.0490",
                "balance": "459567.9171"
            }
        }
    });

    Mock::given(method("POST"))
        .and(path("/0/private/QueryLedgers"))
        .and(body_string_contains("id=L4UESK-KG3EQ-UFO4T5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(&server)
        .await;

    let client = build_client(&server);
    let request = QueryLedgersRequest::new("L4UESK-KG3EQ-UFO4T5");
    let ledgers = client.query_ledgers(&request).await.unwrap();

    assert_eq!(ledgers.len(), 1);
    assert_eq!(ledgers["L4UESK-KG3EQ-UFO4T5"].asset, "ZGBP");
}

#[tokio::test]
async fn test_get_order_amends() {
    let server = MockServer::start().await;
    let response = serde_json::json!({
        "error": [],
        "result": {
            "amends": [{
                "amend_id": "TYNQqQ-AY6Py-o5PJoM",
                "amend_type": "user",
                "order_qty": "5.25",
                "remaining_qty": "5.25",
                "limit_price": "0.30",
                "post_only": false,
                "timestamp": 1728821182545u64
            }],
            "count": 1
        }
    });

    Mock::given(method("POST"))
        .and(path("/0/private/OrderAmends"))
        .and(body_string_contains("\"order_id\":\"OVM3PT-56ACO-53SM2T\""))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(&server)
        .await;

    let client = build_client(&server);
    let request = OrderAmendsRequest::new("OVM3PT-56ACO-53SM2T");
    let amends = client.get_order_amends(&request).await.unwrap();

    assert_eq!(amends.count, 1);
    assert_eq!(amends.amends[0].amend_id, "TYNQqQ-AY6Py-o5PJoM");
}

#[tokio::test]
async fn test_export_report_lifecycle() {
    let server = MockServer::start().await;

    let add_response = serde_json::json!({
        "error": [],
        "result": { "id": "TCJA" }
    });
    Mock::given(method("POST"))
        .and(path("/0/private/AddExport"))
        .and(body_string_contains("report=ledgers"))
        .respond_with(ResponseTemplate::new(200).set_body_json(add_response))
        .mount(&server)
        .await;

    let status_response = serde_json::json!({
        "error": [],
        "result": [{
            "id": "TCJA",
            "descr": "my_trades_1",
            "format": "CSV",
            "report": "ledgers",
            "status": "Processed"
        }]
    });
    Mock::given(method("POST"))
        .and(path("/0/private/ExportStatus"))
        .respond_with(ResponseTemplate::new(200).set_body_json(status_response))
        .mount(&server)
        .await;

    let report_bytes = b"PK\x03\x04-fake-zip-content".to_vec();
    Mock::given(method("POST"))
        .and(path("/0/private/RetrieveExport"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(report_bytes.clone()))
        .mount(&server)
        .await;

    let remove_response = serde_json::json!({
        "error": [],
        "result": { "delete": true }
    });
    Mock::given(method("POST"))
        .and(path("/0/private/RemoveExport"))
        .and(body_string_contains("type=delete"))
        .respond_with(ResponseTemplate::new(200).set_body_json(remove_response))
        .mount(&server)
        .await;

    let client = build_client(&server);

    let added = client
        .add_export(&AddExportRequest::new(ReportType::Ledgers, "my_trades_1"))
        .await
        .unwrap();
    assert_eq!(added.id, "TCJA");

    let statuses = client
        .get_export_status(&ExportStatusRequest::new(ReportType::Ledgers))
        .await
        .unwrap();
    assert_eq!(statuses[0].status, "Processed");

    let data = client
        .retrieve_export(&RetrieveExportRequest::new("TCJA"))
        .await
        .unwrap();
    assert_eq!(data, report_bytes);

    let removed = client
        .remove_export(&RemoveExportRequest::new("TCJA", RemoveExportType::Delete))
        .await
        .unwrap();
    assert_eq!(removed.delete, Some(true));
}

#[tokio::test]
async fn test_create_subaccount_and_transfer() {
    let server = MockServer::start().await;

    let create_response = serde_json::json!({
        "error": [],
        "result": true
    });
    Mock::given(method("POST"))
        .and(path("/0/private/CreateSubaccount"))
        .and(body_string_contains("username=fred"))
        .respond_with(ResponseTemplate::new(200).set_body_json(create_response))
        .mount(&server)
        .await;

    let transfer_response = serde_json::json!({
        "error": [],
        "result": {
            "transfer_id": "TOH3AS2-LPCWR8-JDQGEU",
            "status": "complete"
        }
    });
    Mock::given(method("POST"))
        .and(path("/0/private/AccountTransfer"))
        .and(body_string_contains("asset=XBT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(transfer_response))
        .mount(&server)
        .await;

    let client = build_client(&server);

    let created = client
        .create_subaccount(&CreateSubaccountRequest::new("fred", "fred@example.com"))
        .await
        .unwrap();
    assert!(created);

    let transfer = client
        .account_transfer(&AccountTransferRequest::new(
            "XBT",
            Decimal::from_str("1.0").unwrap(),
            "ABCD 1234 EFGH 5678",
            "IJKL 0987 MNOP 6543",
        ))
        .await
        .unwrap();
    assert_eq!(transfer.transfer_id, "TOH3AS2-LPCWR8-JDQGEU");
}
