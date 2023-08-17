#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Exchange {
    Binance = 0,
    Bitstamp = 1,
}

impl ToString for Exchange {
    fn to_string(&self) -> String {
        match self {
            Exchange::Binance => "Binance".to_string(),
            Exchange::Bitstamp => "Bitstamp".to_string(),
        }
    }
}

pub mod orderbook_aggregator {
    tonic::include_proto!("orderbook"); // The string specified here must match the proto package name
}

pub use orderbook_aggregator::{Empty, Level, Summary};

impl Summary {
    #[allow(dead_code)]
    pub fn merge(ob1: OrderBook, ob2: OrderBook, best_of: usize) -> Summary {
        let mut bids = ob1.bids.clone();
        bids.extend(ob2.bids.clone());
        bids.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap());
        bids.truncate(best_of);

        let mut asks = ob1.asks.clone();
        asks.extend(ob2.asks.clone());
        asks.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());
        asks.truncate(best_of);

        Summary {
            spread: asks[0].price - bids[0].price,
            bids,
            asks,
        }
    }
}

pub use orderbook_aggregator::orderbook_aggregator_client::OrderbookAggregatorClient;
pub use orderbook_aggregator::orderbook_aggregator_server::{
    OrderbookAggregator, OrderbookAggregatorServer,
};

#[derive(Debug, Clone)]
pub struct OrderBook {
    pub exchange: Exchange,
    pub last_updated: String,
    pub bids: Vec<Level>,
    pub asks: Vec<Level>,
}
