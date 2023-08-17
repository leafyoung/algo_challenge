use tokio::sync::broadcast;
use tokio::sync::mpsc;

use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

use crate::types::{Empty, OrderbookAggregator, Summary};

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
                    tx.send(Ok(ob)).await.unwrap();
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
