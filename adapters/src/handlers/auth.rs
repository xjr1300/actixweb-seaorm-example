use actix_web::{web, HttpResponse, Responder};
use serde_json::json;

use usecases::{
    auth::{Credential, ErrorKind},
    database_service::DatabaseService,
};

/// 有効期限付きアクセス・リフレッシュトークンを取得する。
///
/// # Arguments
///
/// * `repos` - リポジトリエクステンション。
/// * `credential` - Eメールとパスワードを格納したクレデンシャル。
///
/// ```bash
/// curl --include --request POST --header "Content-Type: application/json" --data '{"email": "foo@example.com", "password": "012abcEFG=+"}' http://127.0.0.1:8000/auth/obtain_tokens
/// ```
pub async fn obtain_tokens(
    db_service: web::Data<dyn DatabaseService>,
    credential: web::Json<Credential>,
) -> impl Responder {
    match usecases::auth::obtain_tokens(db_service.as_ref(), credential.into_inner()).await {
        Ok(tokens) => HttpResponse::Ok().json(tokens),
        Err(err) => {
            let mut response = match err.code {
                ErrorKind::InternalServerError => HttpResponse::InternalServerError(),
                _ => HttpResponse::BadRequest(),
            };
            response.json(json!({"message": err.message }))
        }
    }
}
