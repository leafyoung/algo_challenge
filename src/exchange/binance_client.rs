//! <https://github.com/binance/binance-spot-api-docs/blob/master/web-socket-streams.md>

use std::pin::Pin;

use futures::{stream::SplitSink, StreamExt};
use futures_util::{SinkExt, Stream};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use async_stream::stream;
use std::fmt::Debug;
use crate::exchange::error::Error;

pub const SUBSCRIBE_METHOD: &str = "SUBSCRIBE";

/// The order book levels (i.e. depth) to subscribe to).
/// Valid <levels> are 5, 10, or 20
#[allow(dead_code)]
pub enum Levels {
    L5 = 5,
    L10 = 10,
    L20 = 20,
}

#[allow(dead_code)]
pub enum Speed {
    S1000 = 1000,
    S100 = 100,
}

#[derive(Debug, Serialize)]
pub struct Request<D> {
    pub method: String,
    pub params: D,
    pub id: u64,
}


#[derive(Debug, Deserialize)]
pub struct BookEvent {
    #[serde(rename = "lastUpdateId")]
    pub last_update_id: u64,
    pub bids: Vec<(String, String)>,
    pub asks: Vec<(String, String)>,
}

pub type Result<T> = std::result::Result<T, Error>;

// Alternative:
// pub const DEFAULT_WS_BASE_URL: &str = "wss://stream.binance.com:443/ws"
// Direct link: "wss://stream.binance.com:9443/ws/ethbtc@depth10@100ms";
#[allow(dead_code)]
pub const DEFAULT_WS_BASE_URL: &str = "wss://stream.binance.com:9443";
pub const DEFAULT_MARKET_DATA_WS_BASE_URL: &str = "wss://data-stream.binance.vision";

/// A WebSocket client for Binance.
pub struct BinanceClient {
    sender: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    // The thread_handle will be dropped when the Client drops.
    #[allow(dead_code)]
    thread_handle: tokio::task::JoinHandle<()>,
    pub broadcast: Option<tokio::sync::broadcast::Sender<String>>,
    pub book_events: Option<Pin<Box<dyn Stream<Item = BookEvent> + Send + Sync>>>,
    next_id: u64,
}

impl BinanceClient {
    pub async fn connect(url: &str) -> Result<Self> {
        let (stream, _) = connect_async(url).await?;
        let (sender, receiver) = stream.split();
        let (broadcast_sender, _) = tokio::sync::broadcast::channel::<String>(32);

        let broadcast = broadcast_sender.clone();

        let thread_handle = tokio::spawn(async move {
            let mut receiver = receiver;

            while let Some(result) = receiver.next().await {
                if let Ok(msg) = result {
                    if let Message::Text(string) = msg {
                        tracing::debug!("{string}");
                        if let Err(err) = broadcast_sender.send(string) {
                            tracing::trace!("{err:?}");
                            break;
                        }
                    }
                } else {
                    tracing::error!("{:?}", result);
                }
            }
        });

        Ok(Self {
            sender,
            thread_handle,
            broadcast: Some(broadcast),
            book_events: None,
            next_id: 0,
        })
    }

    pub async fn connect_public() -> Result<Self> {
        let url = format!("{DEFAULT_MARKET_DATA_WS_BASE_URL}/ws");
        Self::connect(&url).await
    }

    /// Sends a message to the WebSocket.
    pub async fn send<R>(&mut self, req: R) -> Result<()>
    where
        R: Serialize,
    {
        let msg = serde_json::to_string(&req).unwrap();
        tracing::debug!("{msg}");
        self.sender.send(Message::Text(msg.to_string())).await?;

        Ok(())
    }


    /// Performs a remote procedure call.
    pub async fn call<D>(&mut self, event: impl Into<String>, params: D) -> Result<()>
    where
        D: Debug + Serialize,
    {
        let req = Request {
            method: event.into(),
            params,
            id: self.gen_next_id(),
        };
        self.send(req).await
    }

    fn gen_next_id(&mut self) -> u64 {
        self.next_id += 1;
        self.next_id
    }
    // <https://github.com/binance/binance-spot-api-docs/blob/master/web-socket-streams.md#partial-book-depth-streams>
    pub async fn subscribe_orderbook(&mut self, symbol: &str, levels: Levels, speed: Speed) {
        let topic: String = format!("{symbol}@depth{}@{}ms", levels as u8, speed as u16);

        self.call(SUBSCRIBE_METHOD, vec![topic])
            .await
            .expect("cannot send request");

        let mut messages_receiver = self
            .broadcast
            .clone()
            .expect("client not connected")
            .subscribe();

        let depth_events = stream! {
            while let Ok(msg) = messages_receiver.recv().await {
                if let Ok(msg) = serde_json::from_str::<BookEvent>(&msg) {
                    yield msg
                }
            }
        };

        let depth_events = Box::pin(depth_events);

        self.book_events = Some(depth_events);
    }


}