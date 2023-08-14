
//! <https://www.bitstamp.net/websocket/v2/>

use std::pin::Pin;

use futures::{stream::SplitSink, StreamExt};
use futures_util::{SinkExt, Stream};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use async_stream::stream;

use crate::bitstamp::error::Error;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Serialize)]
pub struct Request<D> {
    pub event: String,
    pub data: D,
}


pub const SUBSCRIBE_EVENT: &str = "bts:subscribe";

#[derive(Default, Serialize)]
pub struct SubscribeData {
    channel: String,
}

impl SubscribeData {
    pub fn new(channel: &str) -> Self {
        Self {
            channel: channel.into(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct BookEvent {
    pub data: BookData,
}

#[derive(Debug, Deserialize)]
pub struct BookData {
    pub timestamp: String,
    pub microtimestamp: String,
    pub bids: Vec<(String, String)>,
    pub asks: Vec<(String, String)>,
}

pub const DEFAULT_WS_BASE_URL: &str = "wss://ws.bitstamp.net";

/// A WebSocket client for Bitstamp.
pub struct Client {
    sender: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    // The thread_handle will be dropped when the Client drops.
    #[allow(dead_code)]
    thread_handle: tokio::task::JoinHandle<()>,
    pub broadcast: Option<tokio::sync::broadcast::Sender<String>>,
    pub book_events: Option<Pin<Box<dyn Stream<Item = BookEvent> + Send + Sync>>>,
}

impl Client {
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
                            // Break the while loop so that the receiver handle is dropped
                            // and the task unsubscribes from the summary stream.
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
        })
    }

    pub async fn connect_public() -> Result<Self> {
        let url = format!("{DEFAULT_WS_BASE_URL}/");
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
    pub async fn call<D>(&mut self, event: impl Into<String>, data: D) -> Result<()>
    where
        D: Serialize,
    {
        let req = Request {
            event: event.into(),
            data,
        };

        self.send(req).await
    }

    // <https://assets.bitstamp.net/static/webapp/examples/order_book_v2.3610acefe104f01a8dcd4fda1cb88403f368526b.html>
    pub async fn subscribe_orderbook(&mut self, symbol: &str) {
        let channel = format!("order_book_{symbol}");

        self.call(SUBSCRIBE_EVENT, SubscribeData { channel })
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