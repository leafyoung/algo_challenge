mod exchange;

use crate::exchange::bitstamp_client::BitstampClient;
use crate::exchange::binance_client::{BinanceClient, Levels, Speed};

use futures::StreamExt;


#[tokio::main]
async fn main() {
    // streaming_bitstamp("btcusd").await;
    // streaming_binance("btcusdt", None, None).await;
    // streaming_binance("btcusdt", None, None);
}



async fn streaming_bitstamp(symbol: &str) {
    let mut bitstamp_client = BitstampClient::connect_public().await.expect("cannot connect");
    bitstamp_client.subscribe_orderbook(&symbol).await;
    let mut book_events = bitstamp_client.book_events.unwrap();
    while let Some(msg) = book_events.next().await {
        // dbg!(&msg);
        dbg!(msg.data.microtimestamp);
        msg.data.asks.iter().for_each(|x| {
            // println!("ask: {:?}", x);
        });
    }

}

async fn streaming_binance(symbol: &str, levels: Option<Levels>, speed: Option<Speed>) {
    let mut binance_client = BinanceClient::connect_public().await.expect("cannot connect");
    binance_client.subscribe_orderbook("btcusdt", levels.unwrap_or(Levels::L20), speed.unwrap_or(Speed::S100)).await;
    let mut depth_events = binance_client.book_events.unwrap();
    while let Some(msg) = depth_events.next().await {
        dbg!(msg.last_update_id);
        // dbg!(&msg);
    }

}