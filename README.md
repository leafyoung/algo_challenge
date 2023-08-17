# Challenge

## Test

Server

```bash
cargo run --release --bin server
```

Client

```bash
cargo run --release --bin client
```

# Tasks

1.  connects to two exchanges' websocket feeds at the same time,
2.  pulls order book or a given traded pair of currencies (configurable), from each exchanges, using these streaming connections,
3.  merges and sorts the order books to create a combined order book
4.  from the combined book, publishes the spread, top ten bids, and top ten asks, as a stream, through a gRPC server.

## TODO

-   [x] Test Binance & Bitstamp with cli
-   [x] Connect to Exchange's Websocket
-   [x] Connect to two exchanges' websocket feeds at the same time
-   [x] Aggregation
-   [x] gRPC server
-   [x] Optimize

## Design

```
BinanceClient  ---Orderbook---\                                        /---> Client A
                              |---> Manager ---Summary---> gRPC Server ---> Client B
BitstampClient ---Orderbook---/                                        \---> Client C
```

-   Summary is merged from two order books. When one of the orderbook gets updated, it will be merged the another and sent to gRPC server.

## Reference

### wscat cli

```bash
wscat -c wss://stream.binance.com:9443/ws/btcusdt@ticker
wscat -c wss://stream.binance.com:9443/ws/ethbtc@depth10@100ms

# 10 levels
wscat -c wss://stream.binance.com:9443/ws/ethbtc@depth20@100ms
```

```bash
wscat -c wss://stream.binance.com:9443/ws/btcusdt
```

```bash
wscat -c wss://stream.binance.com:9443/ws
```

```json
{ "method": "SUBSCRIBE",   "params": [ "btcusdt@depth20@100ms"  ],  "id": 1 }
{ "method": "SUBSCRIBE",   "params": [     "btcusdt@aggTrade",     "btcusdt@depth"  ],  "id": 1 }
{ "method": "UNSUBSCRIBE",   "params": [     "btcusdt@depth"   ],   "id": 312 }
{ "method": "LIST_SUBSCRIPTIONS", "id": 3 }
```

-   Test with Bitstamp with cli

```bash
# 100 levels
wscat -c wss://ws.bitstamp.net
```

```json
{ "event": "bts:subscribe", "data": { "channel": "order_book_btcusd" } }
```

## Docs

1. Docs for Websocket connection:
   https://github.com/binance/binance-spot-api-docs/blob/master/web-socket-streams.md
   Example API feed:
   https://api.binance.com/api/v3/depth?symbol=ETHBTC
   Websocket connection URL for Binance:
   wss://stream.binance.com:9443/ws/ethbtc@depth20@100ms
   wss://stream.binance.com:9443

2. Bitstamp
   Docs: https://www.bitstamp.net/websocket/v2/
   Example API feed: https://www.bitstamp.net/api/v2/order_book/ethbtc/
   Example Websocket usage: https://www.bitstamp.net/s/webapp/examples/order_book_v2.htm
   https://assets.bitstamp.net/static/webapp/examples/order_book_v2.3610acefe104f01a8dcd4fda1cb88403f368526b.html
