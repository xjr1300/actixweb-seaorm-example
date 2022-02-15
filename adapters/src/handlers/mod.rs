use actix_web::{get, HttpResponse, Responder};

/// `Hello world!`を返却する。
#[get("/")]
pub async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}
