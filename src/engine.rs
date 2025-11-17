use std::{cmp::Reverse, sync::Arc};

use crate::{Order, OrderAction, OrderBook, OrderRequest};
use tokio::sync::{Mutex, mpsc};

pub async fn run_engine(
    mut receiver: mpsc::Receiver<OrderRequest>,
    order_book: Arc<Mutex<OrderBook>>,
) {
    while let Some(order) = receiver.recv().await {
        let mut book = order_book.lock().await;
        let order_id = book.next_order_id;
        book.next_order_id += 1;

        match order.side {
            OrderAction::Buy => {
                book.bids
                    .entry(Reverse(order.price))
                    .or_insert_with(Vec::new)
                    .push(Order::from_request(order, order_id));
            }
            OrderAction::Sell => {
                book.asks
                    .entry(order.price)
                    .or_insert_with(Vec::new)
                    .push(Order::from_request(order, order_id));
            }
        };
    }
}
