pub mod orderbook_aggregator {
    tonic::include_proto!("orderbook"); // The string specified here must match the proto package name
}

use orderbook_aggregator::orderbook_aggregator_client::OrderbookAggregatorClient;
use orderbook_aggregator::Empty;
use std::thread;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = OrderbookAggregatorClient::connect("http://[::1]:50051").await?;

    loop {
        let mut stream = client
            .book_summary(tonic::Request::new(Empty {}))
            .await?
            .into_inner();

        while let Some(feature) = stream.message().await? {
            println!("NOTE = {:?}", feature);
        }

        thread::sleep(Duration::from_secs(2));
    }
}
