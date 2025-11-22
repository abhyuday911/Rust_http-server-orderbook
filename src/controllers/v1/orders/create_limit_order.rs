use actix_web::{
    HttpResponse, Responder,
    web::{self},
};
use serde_json::json;
use tokio::sync::oneshot;

use crate::{AppState, EngineMessage, OrderAction, OrderRequest};

pub async fn create_limit_order(
    state: web::Data<AppState>,
    order: web::Json<OrderRequest>,
) -> impl Responder {
    let trade_sender = state.trades_sender.clone();
    let order = order.into_inner();

    let (os_sender, os_receiver) = oneshot::channel();

    match order.side {
        OrderAction::Buy => println!("its of type Buy"),
        OrderAction::Sell => println!("its of sell type"),
    }

    let msg = EngineMessage {
        payload: order,
        engine_oneshot_sender: os_sender,
    };

    if let Err(_) = trade_sender.send(msg).await {
        println!("receiver fropped");
        return HttpResponse::InternalServerError()
            .json(json!({"message": "something went wrong order not placed"}));
    };

    match os_receiver.await {
        Ok(v) => HttpResponse::Ok().json(json!({"orderId": v})),
        Err(_) => HttpResponse::InternalServerError()
            .json(json!({"message": "aji dikkat aagyi, oneshot receiver failed"})),
    }
}
