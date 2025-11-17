use actix_web::{HttpResponse, Responder, web};
use serde_json::json;

use crate::AppState;

pub async fn get_orderbook(state: web::Data<AppState>) -> impl Responder {
    let order_book = state.order_book.lock().await;

    dbg!(&order_book.asks);
    dbg!(&order_book.bids);
    dbg!(&order_book.next_order_id);
    HttpResponse::Ok().json(json!({"bids": order_book.bids, "asks": order_book.asks}))
}
