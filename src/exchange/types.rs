use serde::{Deserialize, Serialize};

#[derive(Debug)]
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

#[derive(Debug)]
pub struct OrderBook {
    pub exchange: Exchange,
    pub last_updated: String,
    // pub bids: Vec<(f32, f32)>,
    // pub asks: Vec<(f32, f32)>,
    // pub bids: Vec<(String, String)>,
    // pub asks: Vec<(String, String)>,
    pub bids: Vec<Level>,
    pub asks: Vec<Level>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub spread: f32,
    pub bids: Vec<Level>,
    pub asks: Vec<Level>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Level {
    pub exchange: String,
    pub price: f32,
    pub amount: f32,
}

impl Summary {
    pub fn merge(ob1: &OrderBook, ob2: &OrderBook, best_of: usize) -> Summary {
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
