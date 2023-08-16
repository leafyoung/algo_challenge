mod exchange;

use crate::exchange::binance_client::{BinanceClient, PriceLevels, Speed};
use crate::exchange::bitstamp_client::BitstampClient;
use crate::exchange::types::{Exchange, OrderBook, Summary};
use futures::StreamExt;
use std::env;
use tokio::sync::mpsc;

const BEST_OF: usize = 10;

// cargo run --release btcusd btcusdt

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    println!(
        "Usage: {} <symbol_for_bitstamp> <symbol_for_binance>",
        args[0]
    );
    let (bitstamp_symbol, binance_symbol) = if args.len() < 3 {
        println!("Using default symbols: ethbtc ethbtc");
        // (String::from("ethbtc"), String::from("ethbtc"))
        (String::from("btcusdt"), String::from("btcusdt"))
    } else {
        println!("Using symbols: {:?} {:?}", args[1], args[2]);
        (String::from(&args[1]), String::from(&args[2]))
    };

    let (tx, mut rx) = mpsc::channel(32);
    let tx2 = tx.clone();

    let bitstamp_handle =
        tokio::spawn(async move { streaming_bitstamp(&bitstamp_symbol, tx).await });

    let binance_handle =
        tokio::spawn(async move { streaming_binance(&binance_symbol, None, None, tx2).await });

    let manager = tokio::spawn(async move {
        // Start receiving messages
        let mut ob_bitstamp = OrderBook {
            exchange: Exchange::Bitstamp,
            last_updated: String::new(),
            bids: Vec::new(),
            asks: Vec::new(),
        };
        let mut ob_binance = OrderBook {
            exchange: Exchange::Binance,
            last_updated: String::new(),
            bids: Vec::new(),
            asks: Vec::new(),
        };
        while let Some(ob) = rx.recv().await {
            println!("GOT_manager = {:?}", ob.last_updated);
            match ob.exchange {
                Exchange::Binance => {
                    ob_binance = ob;
                }
                Exchange::Bitstamp => {
                    ob_bitstamp = ob;
                }
            }
            dbg!("binance", &ob_binance.bids, &ob_binance.asks);
            dbg!("bitstamp", &ob_bitstamp.bids, &ob_bitstamp.asks);
            let ob_merged = Summary::merge(&ob_bitstamp, &ob_binance, BEST_OF);
            // println!("ob_merged = {:?}",);
            dbg!(ob_merged);
        }
    });

    manager.await.unwrap();
    bitstamp_handle.await.unwrap();
    binance_handle.await.unwrap();
}

async fn streaming_bitstamp(symbol: &str, tx: mpsc::Sender<OrderBook>) {
    let mut bitstamp_client = BitstampClient::connect_public()
        .await
        .expect("cannot connect");
    bitstamp_client.subscribe_orderbook(&symbol, BEST_OF).await;
    let mut book_events = bitstamp_client.book_events.unwrap();
    while let Some(ob) = book_events.next().await {
        tx.send(ob).await.unwrap();
        // println!("GOT = {:?}", ob.last_updated);
    }
}

async fn streaming_binance(
    symbol: &str,
    levels: Option<PriceLevels>,
    speed: Option<Speed>,
    tx: mpsc::Sender<OrderBook>,
) {
    let mut binance_client = BinanceClient::connect_public()
        .await
        .expect("cannot connect");
    binance_client
        .subscribe_orderbook(
            &symbol,
            levels.unwrap_or(PriceLevels::L20),
            speed.unwrap_or(Speed::S100),
            BEST_OF,
        )
        .await;
    let mut depth_events = binance_client.book_events.unwrap();
    while let Some(ob) = depth_events.next().await {
        tx.send(ob).await.unwrap();
        //println!("GOT = {:?}", ob.last_updated);
    }
}
