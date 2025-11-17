use actix_web::{
    HttpResponse, Responder,
    web::{self},
};
use serde_json::json;

use crate::{AppState, OrderAction, OrderRequest};

pub async fn create_limit_order(
    state: web::Data<AppState>,
    order: web::Json<OrderRequest>,
) -> impl Responder {
    let sender = state.trades_sender.clone();
    let order = order.into_inner();

    match order.side {
        OrderAction::Buy => println!("its of type Buy"),
        OrderAction::Sell => println!("its of sell type"),
    }

    if let Err(_) = sender.send(order.clone()).await {
        println!("receiver fropped");
        return HttpResponse::Conflict()
            .json(json!({"error": "something went wrong order not placed"}));
    };

    HttpResponse::Ok().json(json!(order))
}
