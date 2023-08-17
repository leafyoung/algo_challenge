mod exchange;
mod grpc;
mod streaming;
mod types;

use grpc::OrderbookAggregatorService;
use std::env;
use tokio::sync::broadcast;
use tonic::transport::Server;
use types::{Exchange, OrderBook, OrderbookAggregatorServer, Summary};

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

    let (tx1, mut rx) = broadcast::channel::<OrderBook>(32);
    let tx2 = tx1.clone();

    let bitstamp_handle =
        tokio::spawn(async move { streaming::bitstamp(&bitstamp_symbol, tx1, BEST_OF).await });

    let binance_handle =
        tokio::spawn(
            async move { streaming::binance(&binance_symbol, None, None, tx2, BEST_OF).await },
        );

    let (s_tx, mut _s_rx) = broadcast::channel::<Summary>(32);
    let s_tx_clone = s_tx.clone();

    let server = tokio::spawn(async move {
        let addr = SERVER.parse().unwrap();
        let oas = OrderbookAggregatorService {
            s_tx: Some(s_tx_clone),
        };

        Server::builder()
            .add_service(OrderbookAggregatorServer::new(oas))
            .serve(addr)
            .await
    });

    let manager = tokio::spawn(async move {
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
        while let Ok(ob) = rx.recv().await {
            // println!("GOT_manager = {:?}", ob.last_updated);
            match ob.exchange {
                Exchange::Binance => {
                    ob_binance = ob;
                }
                Exchange::Bitstamp => {
                    ob_bitstamp = ob;
                }
            }
            let ob_merged = Summary::merge(ob_bitstamp.clone(), ob_binance.clone(), BEST_OF);

            // println!("{} {}", &ob_merged.bids.len(), &ob_merged.asks.len());
            if let Err(e) = s_tx.send(ob_merged) {
                eprintln!("Error sending message: {}", e);
            }
        }
    });

    manager.await.unwrap();
    bitstamp_handle.await.unwrap();
    binance_handle.await.unwrap();
    let _ = server.await.unwrap();
}
