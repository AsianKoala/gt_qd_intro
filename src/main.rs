mod models;
mod orderbook;
mod server;
mod websocket;
mod listener;

use gt_qd_orderbook::{listener::Listener, orderbook::OrderBook};

#[tokio::main]
async fn main() {
    let cfg = Listener::build_cfg().await.unwrap();
    let mut l = Listener::new(cfg);
    let mut ob = OrderBook::new();
    l.run(&mut ob);
}
