use crate::{EngineMessage, Order, OrderAction, OrderBook, OrderKind};
use serde::Serialize;
use serde_json::to_string_pretty;
use std::{cmp::Reverse, sync::Arc};
use tokio::sync::{Mutex, mpsc};

// this file seems to have a lot of redundancy.

#[derive(Debug, Serialize)]
pub enum EngineReply {
    // market order reply cases.
    PartiallySettled(u16, u32), // qty and average price for it.
    FullySettled(u16, u32),     // qty   // average price
    CompletelyRejected,

    // limit order cases
    AddedToOrderBook(u32),
}

pub async fn run_engine(
    mut receiver: mpsc::Receiver<EngineMessage>,
    order_book: Arc<Mutex<OrderBook>>,
) {
    while let Some(msg) = receiver.recv().await {
        let mut book = order_book.lock().await;

        match msg.payload.order_kind {
            OrderKind::Market => {
                let mut qty = msg.payload.amount.clone();

                match msg.payload.side {
                    OrderAction::Buy => {
                        while qty > 0 {
                            dbg!("inside the buy order");

                            let price_level_option = book.asks.keys().next().cloned();
                            let price_level = match price_level_option {
                                Some(val) => val,
                                // reply here
                                None => break,
                            };
                            dbg!("indie teh buy order");
                            match book.asks.iter_mut().next() {
                                Some(val) => {
                                    let (price_level, orders_at_the_level) = val;
                                    dbg!("inside the match of the case some");
                                    // I think should put this below block of removing the arder from vec;
                                    // is this even possible based on the way the code has been written
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
                    let reply;

                    if qty == 0 {
                        reply = EngineReply::FullySettled(msg.payload.amount, 000)
                    } else if qty < msg.payload.amount {
                        reply = EngineReply::PartiallySettled(msg.payload.amount - qty, 0000)
                    } else {
                        reply = EngineReply::CompletelyRejected
                    }

                    if let Err(_) = msg.engine_oneshot_sender.send(reply) {
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
                    if let Err(_) = msg
                        .engine_oneshot_sender
                        .send(EngineReply::AddedToOrderBook(order_id))
                    {
                        println!("receiver dropped");
                    }
                });
            }
        }
    }
}
