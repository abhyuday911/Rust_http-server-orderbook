use actix_web::{HttpResponse, Responder};

pub async fn index() -> impl Responder {
    HttpResponse::Ok().body("welcome to the / route")
}

pub async fn sign_up() -> impl Responder {
    HttpResponse::Ok().body("hello world")
}

pub async fn sign_in() -> impl Responder {
    HttpResponse::Ok().body("sign_in route")
}
