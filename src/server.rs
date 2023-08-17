mod exchange;
mod streaming;

use crate::exchange::types::{Exchange, OrderBook, Summary};
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
        tokio::spawn(async move { streaming::bitstamp(&bitstamp_symbol, tx, BEST_OF).await });

    let binance_handle =
        tokio::spawn(
            async move { streaming::binance(&binance_symbol, None, None, tx2, BEST_OF).await },
        );

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
