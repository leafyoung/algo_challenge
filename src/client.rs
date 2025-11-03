mod types;

use std::thread;
use std::time::Duration;

use types::{Empty, OrderbookAggregatorClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = OrderbookAggregatorClient::connect("http://[::1]:50051").await?;

    loop {
        let mut stream = client
            .book_summary(tonic::Request::new(Empty {}))
            .await?
            .into_inner();

        while let Some(feature) = stream.message().await? {
            println!("Response = {:?}", feature);
        }

        thread::sleep(Duration::from_secs(2));
    }
}
