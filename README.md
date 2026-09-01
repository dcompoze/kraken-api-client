# kraken-api-client

`kraken-api-client` is an async Rust library for Kraken Spot and Futures APIs:

- Spot REST client for public and private endpoints.
- Spot WebSocket v2 client for market data and trading channels.
- Futures REST and WebSocket clients.
- Typed models for requests and responses.
- Auth support for signatures, credentials providers, and nonce generation.
- Rate limiting utilities for public, private, and trading flows.

## Library

Spot public REST request:

```rust
use kraken_api_client::spot::rest::SpotRestClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SpotRestClient::new()?;
    let time = client.get_server_time().await?;
    println!("Server time: {:?}", time);
    Ok(())
}
```

Spot private REST client with environment credentials:

```rust
use std::sync::Arc;

use kraken_api_client::auth::EnvCredentials;
use kraken_api_client::spot::rest::SpotRestClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let creds = Arc::new(EnvCredentials::from_env());
    let client = SpotRestClient::builder().credentials(creds).build()?;

    let balances = client.get_account_balance().await?;
    println!("Balances: {:?}", balances);
    Ok(())
}
```

Futures public REST request:

```rust
use kraken_api_client::futures::rest::FuturesRestClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = FuturesRestClient::new()?;
    let instruments = client.get_instruments().await?;
    println!("Instruments: {}", instruments.instruments.len());
    Ok(())
}
```

Spot WebSocket market data stream:

```rust
use futures_util::StreamExt;
use kraken_api_client::spot::ws::SpotWsClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ws = SpotWsClient::connect().await?;
    ws.subscribe_ticker(["BTC/USD"]).await?;

    while let Some(msg) = ws.next().await {
        println!("{:?}", msg?);
    }

    Ok(())
}
```

## API coverage

| API | Supported | Endpoints |
|---|---|---|
| Spot REST (public) | ✓ | Time, SystemStatus, Assets, AssetPairs, Ticker, OHLC, Depth, Trades, Spread |
| Spot REST (account) | ✓ | Balance, BalanceEx, TradeBalance, OpenOrders, ClosedOrders, QueryOrders, OrderAmends, TradesHistory, QueryTrades, OpenPositions, Ledgers, QueryLedgers, TradeVolume, exports |
| Spot REST (trading) | ✓ | AddOrder, AddOrderBatch, AmendOrder, EditOrder, CancelOrder, CancelAll, CancelAllOrdersAfter, CancelOrderBatch |
| Spot REST (funding) | ✓ | Deposit and withdrawal methods, addresses, statuses, Withdraw, WithdrawCancel, WalletTransfer |
| Spot REST (sub-accounts) | ✓ | CreateSubaccount, AccountTransfer |
| Spot REST (earn) | ✓ | Allocate, Deallocate, allocation and deallocation status, Strategies, Allocations |
| Spot WebSocket v2 (market data) | ✓ | ticker, book, level3, trade, ohlc, instrument channels |
| Spot WebSocket v2 (user data) | ✓ | executions, balances channels (via GetWebSocketsToken) |
| Spot WebSocket v2 (trading) | ✓ | add_order, cancel_order, cancel_all, edit_order, amend_order, batch_add, batch_cancel, cancel_all_orders_after |
| Futures REST (public) | ✓ | tickers, orderbook, history, instruments, instrument status, fee schedules, historical funding rates, charts |
| Futures REST (private) | ✓ | accounts, open positions, open orders, fills, send/edit/cancel order, batch orders, transfers, withdrawal, account log, notifications, leverage and PnL preferences, unwind queue |
| Futures REST (sub-accounts) | ✓ | list sub-accounts with holding, futures, and flex account balances |
| Futures REST (market history) | ✓ | historical order, execution, and trigger events, account log CSV export |
| Futures WebSocket | ✓ | Public and private feed subscriptions with challenge authentication |

## TLS backends

Rustls is the default backend for HTTPS and WSS connections.

To use the platform-native TLS backend, disable default features and enable `native-tls`.

```toml
[dependencies]
kraken-api-client = { version = "1.2", default-features = false, features = ["native-tls"] }
```

## Credentials

Private endpoints use credentials from your own source.

You can use `StaticCredentials` or `EnvCredentials`.

Expected env vars are `KRAKEN_API_KEY` and `KRAKEN_API_SECRET`.

## Project structure

```text
.
├── src/
│   ├── auth/                 # API key/secret handling, signing, and nonce utilities.
│   ├── rate_limit/           # Client-side rate-limit policies and helpers.
│   ├── types/                # Common reusable data types and serde helpers.
│   ├── spot/                 # Kraken Spot REST + WebSocket modules.
│   │   ├── rest/             # Spot REST client, endpoint routing, and typed payloads.
│   │   └── ws/               # Spot WS client, stream handling, and channel messages.
│   └── futures/              # Kraken Futures REST + WebSocket modules.
│       ├── rest/             # Futures REST client, endpoints, and response models.
│       └── ws/               # Futures WS client, stream logic, and WS message types.
├── examples/                 # Runnable examples for public/private REST and WS endpoints.
└── tests/                    # Integration and smoke tests.
```
