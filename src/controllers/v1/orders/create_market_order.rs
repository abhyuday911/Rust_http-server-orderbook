use actix_web::{HttpResponse, Responder, web};
use serde_json::json;
use tokio::sync::oneshot;

use crate::{AppState, EngineMessage, OrderKind, OrderRequest};

pub async fn create_market_order(
    order: web::Json<OrderRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let order = order.into_inner();
    let trade_sender = state.trades_sender.clone();

    // check if correct order_kind.
    match order.order_kind {
        OrderKind::Market => {}
        _ => {
            HttpResponse::UnprocessableEntity()
                .json(json!({"message" : "Didn't send correct OrderKind"}));
        }
    }

    let (os_sender, os_receiver) = oneshot::channel();
    let msg = EngineMessage {
        payload: order,
        engine_oneshot_sender: os_sender,
    };

    if let Err(_) = trade_sender.send(msg).await {
        println!("receiver dropped");
        return HttpResponse::InternalServerError().json(json!({"message": "receiver dropped"}));
    }

    match os_receiver.await {
        Ok(order_id) => HttpResponse::Ok().json(json!({"orderId" : order_id})),
        Err(_) => HttpResponse::InternalServerError()
            .json(json!({ "message" : "didn't receive anything from oneshot"})),
    }
}
