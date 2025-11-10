use crate::{AppState, User};
use actix_web::{
    HttpResponse, Responder,
    web::{self, Data},
};
use serde_json::json;

pub async fn index() -> impl Responder {
    HttpResponse::Ok().body("welcome to the / route")
}

pub async fn sign_up(state: Data<AppState>, user_data: web::Json<User>) -> impl Responder {
    let users = state.users.lock().unwrap();

    // check if user exists, if yes bhag yha se ****
    // hash the user data
    // push it into the users hashmap
    // set cookie
    // send back response

    HttpResponse::Ok().json(json!(*users))
}

pub async fn sign_in() -> impl Responder {
    HttpResponse::Ok().body("sign_in route")
}
