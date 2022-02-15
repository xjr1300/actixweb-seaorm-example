use actix_web::{web, HttpResponse, Responder};
use serde_json::json;

use usecases::database_service::DatabaseService;
use usecases::prefectures;

pub async fn list(db_service: web::Data<dyn DatabaseService>) -> impl Responder {
    match prefectures::list(db_service.as_ref()).await {
        Ok(prefectures) => HttpResponse::Ok().json(prefectures),
        Err(err) => {
            HttpResponse::InternalServerError().json(json!({ "message": format!("{}", err) }))
        }
    }
}
