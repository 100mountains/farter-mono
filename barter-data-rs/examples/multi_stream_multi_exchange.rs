use barter_data::{
    event::{DataKind, MarketEvent},
    exchange::{
        binance::{futures::BinanceFuturesUsd, spot::BinanceSpot},
        kraken::Kraken,
        okx::Okx,
    },
    streams::Streams,
    subscription::{
        book::{OrderBooksL1, OrderBooksL2},
        trade::PublicTrades,
    },
};
use barter_integration::model::instrument::kind::InstrumentKind;
use tokio_stream::StreamExt;
use tracing::info;

// this part does wgat ? 
#[rustfmt::skip]
#[tokio::main]
/**
 * The `main` function is an asynchronous Rust function that initializes and runs a program for subscribing to
 * market event streams from various exchanges. Here is a breakdown of the code:
 * 
 * 1. The `init_logging` function is called to initialize the logging system for tracing logs.
 * 
 * 2. The `Streams` struct is created using the `Streams::builder_multi()` method, which allows building multiple
 * streams for different types of market events.
 * 
 * 3. Public trade streams are added to the `Streams` object using the `add` method and
 * `Streams::<PublicTrades>::builder()`.
 * 
 * 4. Multiple subscriptions for public trades are added using the `subscribe` method. Each subscription consists
 * of a tuple containing the exchange, trading pair, instrument kind, and the type of market event (in this
 * case, `PublicTrades`).
 * 
 * 5. Similarly, order book level 1 streams and order book level 2 streams are added to the `Streams` object
 * using the `add` method and respective builders.
 * 
 * 6. The `init` method is called on the `Streams` object to initialize the streams asynchronously.
 * 
 * 7. The `join_map` method is called on the `Streams` object to join all the exchange streams into a single
 * `tokio_stream::StreamMap`.
 * 
 * 8. A loop is started to iterate over the joined stream. The `next` method is called on the `joined_stream` to
 * await the next market event from any exchange.
 * 
 * 9. Inside the loop, the exchange ID and the market event data are extracted from the stream using pattern
 * matching.
 * 
 * 10. The exchange ID and market event data are logged using the `info!` macro.
 * 
 * This code demonstrates how to subscribe to market event streams from multiple exchanges and process the
 * received data asynchronously.
 */
async fn main() {
    // Initialise INFO Tracing log subscriber
    init_logging();

    // Notes:
    // - MarketEvent<DataKind> could use a custom enumeration if more flexibility is required.
    // - Each call to StreamBuilder::subscribe() creates a separate WebSocket connection for those
    //   Subscriptions passed.

    // Initialise MarketEvent<DataKind> Streams for various exchanges
    let streams: Streams<MarketEvent<DataKind>> = Streams::builder_multi()

        // Add PublicTrades Streams for various exchanges
        .add(Streams::<PublicTrades>::builder()
            .subscribe([
                (BinanceSpot::default(), "btc", "usdt", InstrumentKind::Spot, PublicTrades),
            ])
            .subscribe([
                (BinanceFuturesUsd::default(), "btc", "usdt", InstrumentKind::Perpetual, PublicTrades),
            ])
            .subscribe([
                (Okx, "btc", "usdt", InstrumentKind::Spot, PublicTrades),
                (Okx, "btc", "usdt", InstrumentKind::Perpetual, PublicTrades),
            ])
        )

        // Add OrderBooksL1 Stream for various exchanges
        .add(Streams::<OrderBooksL1>::builder()
            .subscribe([
                (BinanceSpot::default(), "btc", "usdt", InstrumentKind::Spot, OrderBooksL1),
            ])
            .subscribe([
                (BinanceFuturesUsd::default(), "btc", "usdt", InstrumentKind::Perpetual, OrderBooksL1),
            ])
            .subscribe([
                (Kraken, "xbt", "usd", InstrumentKind::Spot, OrderBooksL1),
            ])
        )

        // Add OrderBooksL2 Stream for various exchanges
        .add(Streams::<OrderBooksL2>::builder()
            .subscribe([
                (BinanceSpot::default(), "btc", "usdt", InstrumentKind::Spot, OrderBooksL2),
            ])
            .subscribe([
                (BinanceFuturesUsd::default(), "btc", "usdt", InstrumentKind::Perpetual, OrderBooksL2),
            ])
        )
        .init()
        .await
        .unwrap();

    // Join all exchange Streams into a single tokio_stream::StreamMap
    // Notes:
    //  - Use `streams.select(ExchangeId)` to interact with the individual exchange streams!
    //  - Use `streams.join()` to join all exchange streams into a single mpsc::UnboundedReceiver!
    let mut joined_stream = streams.join_map().await;

    while let Some((exchange, data)) = joined_stream.next().await {
        info!("Exchange: {exchange}, MarketEvent<DataKind>: {data:?}");
    }
}

// Initialise an INFO `Subscriber` for `Tracing` Json logs and install it as the global default.
fn init_logging() {
    tracing_subscriber::fmt()
        // Filter messages based on the INFO
        .with_env_filter(
            tracing_subscriber::filter::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        // Disable colours on release builds
        .with_ansi(cfg!(debug_assertions))
        // Enable Json formatting
        .json()
        // Install this Tracing subscriber as global default
        .init()
}
