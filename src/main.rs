mod models;
mod orderbook;
mod server;
mod websocket;

use tokio::time::{sleep, Duration};

const MAX_ATTEMPTS: u32 = 5;
const SLEEP_DURATION: Duration = Duration::from_millis(1000);

#[tokio::main]
async fn main() {
    // Initialize WebSocket connection and order book
    // let mut order_book = order_book::OrderBook::new();
    // websocket::start_websocket(&mut order_book);

    let mut attempts = 0;
    let res = loop {
        match server::get_ws_server_info().await {
            Ok(result) => {
                break Ok(result);
            }
            Err(e) => {
                attempts += 1;
                if attempts == MAX_ATTEMPTS {
                    break Err(e);
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

    match res {
        Ok(cfg) => {
            println!("Successfully obtained server info:");
            println!("Token: {}", cfg.token);
            println!("Endpoint: {}", cfg.endpoint);

            let mut order_book = crate::orderbook::OrderBook::new();
            websocket::start_websocket(&cfg, &mut order_book);
        }
        Err(e) => {
            eprintln!(
                "Failed to get WebSocket server info after {} attempts: {}",
                attempts, e
            );
        }
    }
}
