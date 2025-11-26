use actix_web::{HttpResponse, Responder, web};
use serde_json::json;
use tokio::sync::oneshot;

use crate::{AppState, EngineMessage, OrderKind, OrderRequest, engine::EngineReply};

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

    let (os_sender, os_receiver) = oneshot::channel::<EngineReply>();
    let msg = EngineMessage {
        payload: order,
        engine_oneshot_sender: os_sender,
    };

    if let Err(_) = trade_sender.send(msg).await {
        println!("receiver dropped");
        return HttpResponse::InternalServerError().json(json!({"message": "receiver dropped"}));
    }

    match os_receiver.await {
        // if received type is under teh engineReply category
        Ok(message) => {
            match message {
                EngineReply::PartiallySettled(qty, _) => HttpResponse::Ok()
                    .json(json!({"message" : "Order Partially settled", "settled_quantity": qty})),
                EngineReply::FullySettled(qty, _average_price) => HttpResponse::Ok()
                    .json(json!({"message" : "Order Completely Settled", "settled_quantity": qty})),
                EngineReply::CompletelyRejected => {
                    HttpResponse::Ok().json(json!({"message" : "Sorry coudn't fulfill your order, no corresponding trades found"}))
                }
                _ => HttpResponse::InternalServerError().json(json!({"message" : "Bhai kuch to babal hogya yha tk bat aani nhi chaiye thi market order me"})),
            }
        }
        Err(_) => HttpResponse::InternalServerError()
            .json(json!({ "message" : "didn't receive anything from oneshot"})),
    }
}
