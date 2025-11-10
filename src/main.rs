use actix_web::{App, HttpServer, web};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub mod controllers;
use crate::controllers::{index, sign_up};
// user struct
#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    name: String,
    password: String,
    age: u8,
}
// we need this here so throughout heaven and earth all threads are able to access this
#[derive(Clone)]
pub struct AppState {
    users: Arc<Mutex<HashMap<String, User>>>, // hashmap will have key of usename and value will be user details
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let state = web::Data::new(AppState {
        users: Arc::new(Mutex::new(HashMap::new())),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .route("/", web::get().to(index))
            .route("/signup", web::post().to(sign_up))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
