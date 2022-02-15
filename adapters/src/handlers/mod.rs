pub mod prefectures;

use actix_web::{HttpResponse, Responder};

/// `Hello world!`を返却する。
pub async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}
