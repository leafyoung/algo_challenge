use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status};

mod types;

use types::{Empty, Level, OrderbookAggregator, OrderbookAggregatorServer, Summary};

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
    let oas = OrderbookAggregatorService::default();

    Server::builder()
        .add_service(OrderbookAggregatorServer::new(oas))
        .serve(addr)
        .await?;

    Ok(())
}
