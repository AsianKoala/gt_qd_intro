use crate::orderbook::OrderBook;
use crate::{models, server};
use std::thread;
use std::time::{Duration, Instant};
use tungstenite::{connect, Message};
use url::Url;
use std::sync::mpsc::{sync_channel, SyncSender};

mod msgtype {
    pub struct Ping;
    pub struct Pong;
}

pub fn start_websocket(cfg: &server::ServerConfig, order_book: &mut OrderBook) {
    let websocket_url = format!("{}?token={}", cfg.endpoint, cfg.token);
    println!("{}", websocket_url);
    let (mut socket, _response) =
        connect(Url::parse(&websocket_url).unwrap()).expect("Cannot connect to WebSocket");

    println!("Connected to the server.");

    let connect_id = {
        let msg = socket.read_message().expect("Error reading message");

        if let Message::Text(text) = msg {
            let parsed: models::WelcomeMsg = serde_json::from_str(&text).unwrap();
            println!("{:#?}", parsed);
            parsed.id
        } else {
            String::new()
        }
    };

    let ping_message = format!(
        r#"{{
          "id": "{}",
          "type": "ping"
    }}"#,
        connect_id
    );

    let subscribe_message = format!(
        r#"{{
          "id": "{}",
          "type": "subscribe",
          "topic": "/contractMarket/level2Depth5:ETHUSDTM",
          "response": true
    }}"#,
        connect_id
    );

    socket
        .write_message(Message::Text(subscribe_message.to_string()))
        .unwrap();

    let (sender, receiver): (SyncSender<msgtype::Ping>, _) = sync_channel(0);
    let (rev_sender, rev_receiver): (SyncSender<msgtype::Pong>, _) = sync_channel(0);

    let ping_interval = Duration::from_millis(cfg.ping_interval); // Ping every 10 seconds
    let mut last_ping_time = Instant::now();

    thread::spawn(move || {
        loop {
            thread::sleep(ping_interval); // Wait for the interval duration

            let _ = sender.send(msgtype::Ping);
            let _ = rev_receiver.recv();
        }
    });

    loop {
        if let Ok(_) = receiver.try_recv() {
            // Send a Ping message
            let ping_data = format!("Ping at {:?}", last_ping_time.elapsed());
            println!("Sending ping: {}", ping_data);

            socket
                .write_message(Message::Text(ping_message.clone()))
                .unwrap();

            last_ping_time = Instant::now();
            let _ = rev_sender.send(msgtype::Pong);
        }

        let msg = socket.read_message().unwrap();

        if let Message::Text(text) = msg {
            match serde_json::from_str::<models::MarketMsgRoot>(&text) {
                Ok(parsed) => {
                    // If parsing is successful, process the bids and asks
                    order_book.ingest_bids(parsed.data.bids);
                    order_book.ingest_asks(parsed.data.asks);
                    order_book.display();
                }
                Err(e) => {
                    // If parsing fails, log the error and message
                    eprintln!(
                        "Failed to parse message as MarketMsgRoot: {}\nError: {:?}",
                        text, e
                    );
                }
            }
        }
    }
}
