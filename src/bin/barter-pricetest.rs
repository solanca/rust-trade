use barter_data::{
    exchange::{kraken::Kraken, ExchangeId},
    streams::{StreamBuilder, Streams},
    subscription::trade::PublicTrades,
};
use barter_integration::model::instrument::kind::InstrumentKind;
use tracing::info;

#[tokio::main]
async fn main() {
    init_logging();

    let mut streams = Streams::<PublicTrades>::builder()
        .subscribe([
            (Kraken::default(), "AVAX", "USD", InstrumentKind::Spot, PublicTrades),
        ])
        .init()
        .await
        .unwrap();

    let mut kraken_stream = streams
        .select(ExchangeId::Kraken)
        .unwrap();

    while let Some(public_trade) = kraken_stream.recv().await {
        println!("Received public trade: {:?}", public_trade);
        info!("MarketEvent<PublicTrade>: {:?}", public_trade);
    }
}

fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_env("RUST_LOG"))
        .with_ansi(cfg!(debug_assertions))
        .init();
}
