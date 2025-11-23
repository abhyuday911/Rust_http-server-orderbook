use crate::{EngineMessage, Order, OrderAction, OrderBook, OrderKind};
use serde_json::to_string_pretty;
use std::{cmp::Reverse, sync::Arc};
use tokio::sync::{Mutex, mpsc};

// this file seems to have a lot of redundancy.

pub async fn run_engine(
    mut receiver: mpsc::Receiver<EngineMessage>,
    order_book: Arc<Mutex<OrderBook>>,
) {
    while let Some(msg) = receiver.recv().await {
        let mut book = order_book.lock().await;

        match msg.payload.order_kind {
            OrderKind::Market => {
                let mut qty = msg.payload.amount;

                match msg.payload.side {
                    OrderAction::Buy => {
                        while qty > 0 {
                            let price_level_option = book.asks.keys().next().cloned();
                            let price_level = match price_level_option {
                                Some(val) => val,
                                None => break,
                            };

                            match book.asks.iter_mut().next() {
                                Some(val) => {
                                    let (price_level, orders_at_the_level) = val;

                                    // I think should put this below block of removing the arder from vec;
                                    if orders_at_the_level.is_empty() {
                                        let _ = dbg!(&book);
                                        println!(
                                            "break hogya  -->  bhai dekho aaisa hai ab orders bache nhi hai"
                                        );
                                        break;
                                    };
                                    // this guy ⬆️

                                    println!(
                                        "current best level {} and the order array is {}",
                                        price_level,
                                        to_string_pretty(orders_at_the_level).unwrap()
                                    );

                                    if orders_at_the_level[0].amount > qty {
                                        orders_at_the_level[0].amount -= qty;
                                        qty = 0;
                                        dbg!(&orders_at_the_level);
                                    } else {
                                        qty -= orders_at_the_level[0].amount;
                                        orders_at_the_level.remove(0);
                                        dbg!(&orders_at_the_level);
                                    };
                                }
                                None => {
                                    println!(
                                        "bhai dekho aaisa hai ki appka order fulfill nahi kiya jaa sakta hai"
                                    )
                                }
                            }

                            if qty > 0 && book.asks[&price_level].is_empty() {
                                book.asks.remove(&price_level);
                            }
                            println!("quantity in while loop for the time {}", qty);
                            dbg!(&book);
                        }
                    }
                    OrderAction::Sell => {
                        while qty > 0 {
                            let best_price_option = book.bids.keys().next().cloned();
                            let best_price = match best_price_option {
                                Some(v) => v,
                                None => break,
                            };

                            match book.bids.iter_mut().next() {
                                Some(val) => {
                                    let (_, orders_at_level) = val;

                                    if orders_at_level[0].amount > qty {
                                        orders_at_level[0].amount -= qty;
                                        qty = 0;
                                    } else {
                                        qty -= orders_at_level[0].amount;
                                        orders_at_level.remove(0);
                                    }
                                }
                                None => {
                                    println!("dekho bhai aaisa hai apka order nhi de rhe hm ab")
                                }
                            }

                            if qty > 0 && book.bids[&best_price].is_empty() {
                                book.bids.remove(&best_price);
                            }
                            println!("quantity in current iteration {}", qty);
                            dbg!(&book);
                        }
                    }
                }
                tokio::spawn(async move {
                    let send_value;
                    if qty > 0 {
                        send_value = qty
                    } else {
                        send_value = 911;
                    }

                    if let Err(_) = msg.engine_oneshot_sender.send(send_value as u32) {
                        println!("receiver dropped");
                    }
                });
            }
            OrderKind::Limit => {
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
                }
                tokio::spawn(async move {
                    if let Err(_) = msg.engine_oneshot_sender.send(order_id) {
                        println!("receiver dropped");
                    }
                });
            }
        }
    }
}
