/*
#[derive(Clone)]
pub struct AlexandriaApiServer<T>
where
    T: 'static + DataStore + Send + Sync,
{
    data_store: Arc<T>,
}

impl<T> AlexandriaApiServer<T>
where
    T: 'static + DataStore + Send + Sync,
{
    pub fn new(store: T) -> AlexandriaApiServer<T> {
        AlexandriaApiServer {
            data_store: Arc::new(store),
        }
    }
}
*/

use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, Stream};
use tonic::{transport::Server, Request, Response, Status};

pub mod orderbook_aggregator {
    tonic::include_proto!("orderbook"); // The string specified here must match the proto package name
}

use orderbook_aggregator::orderbook_aggregator_server::{
    OrderbookAggregator, OrderbookAggregatorServer,
};
use orderbook_aggregator::{Empty, Level, Summary};

#[derive(Debug, Default)]
pub struct OrderbookAggregatorService {}

#[tonic::async_trait]
impl OrderbookAggregator for OrderbookAggregatorService {
    type BookSummaryStream = ReceiverStream<Result<Summary, Status>>;

    async fn book_summary(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::BookSummaryStream>, Status> {
        // unimplemented!()

        println!("Got a request: {:?}", _request);

        let mut reply = Summary {
            spread: 0.0,
            asks: vec![Level {
                exchange: "Bitstamp".to_string(),
                price: 0.0,
                amount: 0.0,
            }],
            bids: vec![Level {
                exchange: "Binance".to_string(),
                price: 0.0,
                amount: 0.0,
            }],
        };

        let (tx, rx) = mpsc::channel(4);

        tokio::spawn(async move {
            for _ in 0..5 {
                reply.spread += 1.0;
                tx.send(Ok(reply.clone())).await.unwrap();
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let oba = OrderbookAggregatorService::default();

    Server::builder()
        .add_service(OrderbookAggregatorServer::new(oba))
        .serve(addr)
        .await?;

    Ok(())
}
