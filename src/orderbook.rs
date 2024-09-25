use std::collections::BTreeMap;

use crate::models::StringOrU64;

pub struct OrderBook {
    bids: BTreeMap<String, u64>, // Price (as String) -> Quantity (as u64)
    asks: BTreeMap<String, u64>, // Price (as String) -> Quantity (as u64)
    max_levels: usize,           // Limit the number of price levels (5 in this case)
}

impl OrderBook {
    // Initialize the order book with 5 levels of depth
    pub fn new() -> Self {
        OrderBook {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            max_levels: 5,
        }
    }

    fn update_bid(&mut self, price: String, quantity: u64) {
        if quantity == 0 {
            self.bids.remove(&price);
        } else {
            self.bids.insert(price, quantity);
        }
        self.trim_bids();
    }

    fn update_ask(&mut self, price: String, quantity: u64) {
        if quantity == 0 {
            self.asks.remove(&price);
        } else {
            self.asks.insert(price, quantity);
        }
        self.trim_asks();
    }

    // Trim bids to the top 5 levels (highest prices)
    fn trim_bids(&mut self) {
        while self.bids.len() > self.max_levels {
            let lowest_price = self.bids.keys().next().unwrap().to_string();
            self.bids.remove(&lowest_price); // Remove the lowest price
        }
    }

    // Trim asks to the top 5 levels (lowest prices)
    fn trim_asks(&mut self) {
        while self.asks.len() > self.max_levels {
            let highest_price = self.asks.keys().last().unwrap().to_string();
            self.asks.remove(&highest_price); // Remove the highest price
        }
    }

    pub fn ingest_bids(&mut self, new_bids: Vec<Vec<StringOrU64>>) {
        for bid in new_bids.iter() {
            let price = match &bid[0] {
                StringOrU64::Str(s) => s.clone(),
                StringOrU64::U64(f) => f.to_string(),
            };
            let vol = match &bid[1] {
                StringOrU64::Str(s) => s.parse::<u64>().unwrap(),
                StringOrU64::U64(f) => *f,
            };
            self.update_bid(price, vol);
        }
    }

    pub fn ingest_asks(&mut self, new_asks: Vec<Vec<StringOrU64>>) {
        for ask in new_asks.iter() {
            let price = match &ask[0] {
                StringOrU64::Str(s) => s.clone(),
                StringOrU64::U64(f) => f.to_string(),
            };
            let vol = match &ask[1] {
                StringOrU64::Str(s) => s.parse::<u64>().unwrap(),
                StringOrU64::U64(f) => *f,
            };
            self.update_ask(price, vol);
        }
    }

    pub fn display(&self) {
        // Print headers for the table
        println!(
            "\n{:<15} {:<15}   |   {:<15} {:<15}",
            "Bid Price", "Quantity", "Ask Price", "Quantity"
        );
        println!("{:-<60}", ""); // Print a separator

        // Collect the top max_levels orders into vectors to print them together
        let bids: Vec<_> = self.bids.iter().rev().take(self.max_levels).collect(); // Bids in descending order
        let asks: Vec<_> = self.asks.iter().take(self.max_levels).collect(); // Asks in ascending order

        // Iterate through both bids and asks simultaneously and print them side by side
        for i in 0..self.max_levels {
            let bid = bids.get(i);
            let ask = asks.get(i);

            match (bid, ask) {
                (Some((bid_price, &bid_qty)), Some((ask_price, &ask_qty))) => {
                    // Print both bid and ask side by side
                    println!(
                        "{:<15} {:<15}   |   {:<15} {:<15}",
                        bid_price, bid_qty, ask_price, ask_qty
                    );
                }
                (Some((bid_price, &bid_qty)), None) => {
                    // Only bid exists, no ask available
                    println!(
                        "{:<15} {:<15}   |   {:<15} {:<15}",
                        bid_price, bid_qty, "", ""
                    );
                }
                (None, Some((ask_price, &ask_qty))) => {
                    // Only ask exists, no bid available
                    println!(
                        "{:<15} {:<15}   |   {:<15} {:<15}",
                        "", "", ask_price, ask_qty
                    );
                }
                (None, None) => {
                    // Neither bid nor ask exists
                    break;
                }
            }
        }
    }
}
