mod exchange;
mod grpc;
mod streaming;
mod types;

use grpc::{manager, start_grpc_server};
use std::env;
use tokio::sync::broadcast;

use types::{OrderBook, Summary};

const BEST_OF: usize = 10;
const SERVER: &str = "[::1]:50051";

// cargo run --release btcusd btcusdt

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    println!(
        "Usage: {} <symbol_for_bitstamp> <symbol_for_binance>",
        args[0]
    );
    let (bitstamp_symbol, binance_symbol) = if args.len() < 3 {
        println!("Using: default symbols: ethbtc ethbtc");
        // (String::from("ethbtc"), String::from("ethbtc"))
        (String::from("btcusdt"), String::from("btcusdt"))
    } else {
        println!("Using: defined symbols: {:?} {:?}", args[1], args[2]);
        (String::from(&args[1]), String::from(&args[2]))
    };

    let (tx1, rx) = broadcast::channel::<OrderBook>(32);
    let tx2 = tx1.clone();

    let bitstamp_handle =
        tokio::spawn(async move { streaming::bitstamp(&bitstamp_symbol, tx1, BEST_OF).await });

    let binance_handle =
        tokio::spawn(
            async move { streaming::binance(&binance_symbol, None, None, tx2, BEST_OF).await },
        );

    let (s_tx, mut _s_rx) = broadcast::channel::<Summary>(32);
    let s_tx_clone = s_tx.clone();

    let server = tokio::spawn(async move { start_grpc_server(SERVER, s_tx_clone).await });

    let manager = tokio::spawn(async move { manager(rx, s_tx, BEST_OF).await });

    manager.await.unwrap();
    bitstamp_handle.await.unwrap();
    binance_handle.await.unwrap();
    let _ = server.await.unwrap();
}
