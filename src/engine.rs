use std::{cmp::Reverse, sync::Arc};

use crate::{Order, OrderAction, OrderBook};
use serde_json::to_string_pretty;
use tokio::sync::{Mutex, mpsc};

pub async fn run_engine(mut receiver: mpsc::Receiver<Order>, order_book: Arc<Mutex<OrderBook>>) {
    while let Some(order) = receiver.recv().await {
        let mut book = order_book.lock().await;
        book.next_order_id += 1;

        match order.order_action {
            OrderAction::Buy => {
                book.bids
                    .entry(Reverse(order.price))
                    .or_insert_with(Vec::new)
                    .push(order.clone());

                println!(
                    "Bids Book {}",
                    to_string_pretty(&book.bids)
                        .unwrap_or("lg gye, engine ln-19 stringify nhi hua".to_string())
                );
                println!("Asks Book {}", to_string_pretty(&book.asks).unwrap())
            }
            OrderAction::Sell => {
                book.asks
                    .entry(order.price)
                    .or_insert_with(Vec::new)
                    .push(order.clone());
                println!("Asks Book{}", to_string_pretty(&book.asks).unwrap());
                println!("Bids Book{}", to_string_pretty(&book.bids).unwrap())
            }
        };
    }
}
