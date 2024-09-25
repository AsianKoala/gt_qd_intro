use crate::orderbook::OrderBook;
use crate::server::ServerConfig;
use crate::{models, server};
use std::net::TcpStream;
use std::sync::mpsc::{sync_channel, SyncSender};
use std::thread;
use std::time::{Duration, Instant};
use tokio::time::{sleep, Duration as TokioDuration};
use tungstenite::protocol::WebSocketConfig;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{connect, Message};
use url::Url;

mod msgtype {
    pub struct Ping;
    pub struct Pong;
}

pub struct Listener {
    cfg: server::ServerConfig,
    socket: tungstenite::WebSocket<MaybeTlsStream<TcpStream>>,
    connect_id: String,
}

impl Listener {
    pub fn new(cfg: server::ServerConfig) -> Self {
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

        Listener {
            cfg,
            socket,
            connect_id,
        }
    }

    pub fn run(&mut self, order_book: &mut OrderBook) {
        let subscribe_message = format!(
            r#"{{
              "id": "{}",
              "type": "subscribe",
              "topic": "/contractMarket/level2Depth5:ETHUSDTM",
              "response": true
            }}"#,
            self.connect_id
        );

        self.socket
            .write_message(Message::Text(subscribe_message.clone()))
            .unwrap();

        let ping_message = format!(
            r#"{{
              "id": "{}",
              "type": "ping"
            }}"#,
            self.connect_id
        );

        let (sender, receiver): (SyncSender<msgtype::Ping>, _) = sync_channel(0);
        let (rev_sender, rev_receiver): (SyncSender<msgtype::Pong>, _) = sync_channel(0);

        let ping_interval = Duration::from_millis(self.cfg.ping_interval); // Ping every 10 seconds

        thread::spawn(move || {
            loop {
                thread::sleep(ping_interval); // Wait for the interval duration

                let _ = sender.send(msgtype::Ping);
                let _ = rev_receiver.recv();
            }
        });

        loop {
            if let Ok(_) = receiver.try_recv() {
                self.socket
                    .write_message(Message::Text(ping_message.clone()))
                    .unwrap();

                let _ = rev_sender.send(msgtype::Pong);
            }

            let msg = self.socket.read_message().unwrap();

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

    pub async fn build_cfg() -> Result<ServerConfig, String> {
        let mut attempts = 0;
        const MAX_ATTEMPTS: u32 = 5;
        const SLEEP_DURATION: TokioDuration = TokioDuration::from_millis(1000);

        let result = loop {
            match server::get_ws_server_info().await {
                Ok(cfg) => break Ok(cfg),
                Err(e) => {
                    attempts += 1;
                    if attempts == MAX_ATTEMPTS {
                        break Err(format!(
                            "Failed to get server info after {} attempts: {}",
                            attempts, e
                        ));
                    } else {
                        println!(
                            "Failed to get Kucoin server endpoint, retrying ({}/{})",
                            attempts, MAX_ATTEMPTS
                        );
                        sleep(SLEEP_DURATION).await;
                    }
                }
            }
        };

        match result {
            Ok(cfg) => {
                println!("Successfully obtained server info:");
                println!("Token: {}", cfg.token);
                println!("Endpoint: {}", cfg.endpoint);

                Ok(cfg)
            }
            Err(e) => {
                eprintln!("{}", e);
                Err(e)
            }
        }
    }
}
