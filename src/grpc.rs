use tokio::sync::broadcast;
use tokio::sync::mpsc;

use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

use crate::types::{
    Empty, Exchange, OrderBook, OrderbookAggregator, OrderbookAggregatorServer, Summary,
};

pub async fn start_grpc_server(server: &str, s_tx_clone: broadcast::Sender<Summary>) {
    let addr = server.parse().unwrap();
    let oas = OrderbookAggregatorService {
        s_tx: Some(s_tx_clone),
    };

    let _ = Server::builder()
        .add_service(OrderbookAggregatorServer::new(oas))
        .serve(addr)
        .await;
}

pub async fn manager(
    mut rx: broadcast::Receiver<OrderBook>,
    s_tx: broadcast::Sender<Summary>,
    best_of: usize,
) {
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
        let ob_merged = Summary::merge(ob_bitstamp.clone(), ob_binance.clone(), best_of);

        // println!("{} {}", &ob_merged.bids.len(), &ob_merged.asks.len());
        if let Err(e) = s_tx.send(ob_merged) {
            eprintln!("Error sending message: {}", e);
        }
    }
}

#[derive(Debug, Default)]
pub struct OrderbookAggregatorService {
    pub s_tx: Option<broadcast::Sender<Summary>>,
}

#[tonic::async_trait]
impl OrderbookAggregator for OrderbookAggregatorService {
    type BookSummaryStream = ReceiverStream<Result<Summary, Status>>;

    async fn book_summary(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::BookSummaryStream>, Status> {
        println!("Got a request: {:?}", _request);

        let mut s_rx = self.s_tx.clone().expect("not connected").subscribe();

        let (tx, rx) = mpsc::channel(4);
        tokio::spawn(async move {
            loop {
                let data = s_rx.recv().await;
                if let Ok(ob) = data {
                    match tx.send(Ok(ob)).await {
                        Ok(_) => {}
                        Err(_) => {
                            println!("Stopped sending data to gRPC client");
                            break;
                        }
                    };
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
