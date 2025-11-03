use futures::StreamExt;
use tokio::sync::broadcast;

use crate::exchange::binance_client::{BinanceClient, PriceLevels, Speed};
use crate::exchange::bitstamp_client::BitstampClient;
use crate::types::OrderBook;

pub async fn bitstamp(symbol: &str, tx: broadcast::Sender<OrderBook>, best_of: usize) {
    let mut bitstamp_client = BitstampClient::connect_public()
        .await
        .expect("cannot connect");
    bitstamp_client.subscribe_orderbook(symbol, best_of).await;
    let mut book_events = bitstamp_client.book_events.unwrap();
    while let Some(ob) = book_events.next().await {
        tx.send(ob).unwrap();
    }
}

pub async fn binance(
    symbol: &str,
    levels: Option<PriceLevels>,
    speed: Option<Speed>,
    tx: broadcast::Sender<OrderBook>,
    best_of: usize,
) {
    let mut binance_client = BinanceClient::connect_public()
        .await
        .expect("cannot connect");
    binance_client
        .subscribe_orderbook(
            symbol,
            levels.unwrap_or(PriceLevels::L20),
            speed.unwrap_or(Speed::S100),
            best_of,
        )
        .await;
    let mut depth_events = binance_client.book_events.unwrap();
    while let Some(ob) = depth_events.next().await {
        tx.send(ob).unwrap();
    }
}
