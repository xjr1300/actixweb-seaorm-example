use actix_web::{App, HttpServer};

use adapters::handlers::hello;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(hello))
        .bind("127.0.0.1:8000")?
        .run()
        .await
}
