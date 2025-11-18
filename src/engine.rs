use std::{cmp::Reverse, sync::Arc};

use crate::{LimitOrderEngineMessage, Order, OrderAction, OrderBook};
use tokio::sync::{Mutex, mpsc};

pub async fn run_engine(
    mut receiver: mpsc::Receiver<LimitOrderEngineMessage>,
    order_book: Arc<Mutex<OrderBook>>,
) {
    while let Some(msg) = receiver.recv().await {
        let mut book = order_book.lock().await;
        book.next_order_id += 1;
        let order_id = book.next_order_id;

        match msg.payload.side {
            OrderAction::Buy => {
                book.bids
                    .entry(Reverse(msg.payload.price))
                    .or_insert_with(Vec::new)
                    .push(Order::from_request(msg.payload, order_id));
            }
            OrderAction::Sell => {
                book.asks
                    .entry(msg.payload.price)
                    .or_insert_with(Vec::new)
                    .push(Order::from_request(msg.payload, order_id));
            }
        };

        tokio::spawn(async move {
            if let Err(_) = msg.engine_oneshot_sender.send(order_id) {
                println!("receiver dropped");
            }
        });
    }
}
