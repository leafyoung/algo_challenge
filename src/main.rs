mod bitstamp;

use crate::bitstamp::client::Client as BitstampClient;
use crate::bitstamp::client_binance::BinanceClient;

use futures::StreamExt;

#[tokio::main]
async fn main() {

    if true {
        let mut bitstamp_client = BitstampClient::connect_public().await.expect("cannot connect");
        bitstamp_client.subscribe_orderbook("btcusd").await;
        let mut book_events = bitstamp_client.book_events.unwrap();
        while let Some(msg) = book_events.next().await {
            dbg!(&msg);
        }
    } else {
        let mut binance_client = BinanceClient::connect_public().await.expect("cannot connect");
        if false {
            binance_client.subscribe_orderbook("btcusd").await;
            let mut book_events = binance_client.book_events.unwrap();
            while let Some(msg) = book_events.next().await {
                dbg!(&msg);
            }
        }
    }
}