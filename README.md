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
    let client = SpotRestClient::new();
    let time = client.get_server_time().await?;
    println!("Server time: {:?}", time);
    Ok(())
}
```

Spot private REST client with environment credentials:

```rust
use kraken_api_client::auth::credentials::EnvCredentials;
use kraken_api_client::spot::rest::SpotRestClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let creds = EnvCredentials::new("KRAKEN_API_KEY", "KRAKEN_API_SECRET");
    let client = SpotRestClient::with_credentials(creds)?;

    let balances = client.get_account_balances().await?;
    println!("Balances: {:?}", balances);
    Ok(())
}
```

Futures public REST request:

```rust
use kraken_api_client::futures::rest::FuturesRestClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = FuturesRestClient::new();
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
