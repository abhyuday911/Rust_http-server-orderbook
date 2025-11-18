use actix_web::{
    App, HttpServer,
    web::{self},
};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Reverse,
    collections::{BTreeMap, HashMap},
    sync::Arc,
};
use tokio::sync::{Mutex, mpsc, oneshot};

use crate::{
    controllers::v1::{create_limit_order, get_orderbook, index, sign_in, sign_up},
    engine::run_engine,
};
pub mod engine;

pub mod controllers {
    pub mod v1;
}
// user struct
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    username: String,
    name: String,
    password: String,
    age: u8,
}
// we need this here so every one can get a clone of it;
#[derive(Clone, Debug)]
pub struct AppState {
    users: Arc<Mutex<HashMap<String, User>>>, // hashmap will have key of usename and value will be user details
    session_ids: Arc<Mutex<HashMap<String, String>>>,
    trades_sender: mpsc::Sender<LimitOrderEngineMessage>, // type of order. // send oneshot receiver as well
    order_book: Arc<Mutex<OrderBook>>, // arc & mutex -> just in case some other api tries to mutate
}

#[derive(Debug)]
pub struct LimitOrderEngineMessage {
    payload: OrderRequest,
    engine_oneshot_sender: oneshot::Sender<u32>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let (sender, receiver) = mpsc::channel(100);
    let state = web::Data::new(AppState {
        users: Arc::new(Mutex::new(HashMap::new())),
        session_ids: Arc::new(Mutex::new(HashMap::new())),
        trades_sender: sender,
        order_book: Arc::new(Mutex::new(OrderBook {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            next_order_id: 0,
        })),
    });

    tokio::spawn(run_engine(receiver, state.order_book.clone()));

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .route("/", web::get().to(index))
            .route("/signup", web::post().to(sign_up))
            .route("/signin", web::post().to(sign_in))
            .route("/create_limit_order", web::post().to(create_limit_order))
            .route("/get_orderbook", web::get().to(get_orderbook))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Order {
    user_id: String,
    amount: u16,
    asset: String,
    price: u64,
    side: OrderAction,
    order_kind: OrderKind,
    order_id: u32,
}

impl Order {
    fn from_request(req: OrderRequest, order_id: u32) -> Self {
        Order {
            order_id,
            amount: req.amount,
            asset: req.asset,
            price: req.price,
            side: req.side,
            order_kind: req.order_kind,
            user_id: req.user_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderKind {
    Market,
    Limit,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum OrderAction {
    Buy,
    Sell,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderBook {
    bids: BTreeMap<Reverse<u64>, Vec<Order>>,
    asks: BTreeMap<u64, Vec<Order>>,
    next_order_id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRequest {
    user_id: String,
    amount: u16,
    asset: String,
    price: u64,
    side: OrderAction,
    order_kind: OrderKind,
}
