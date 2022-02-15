use actix_web::{web, HttpResponse, Responder};
use serde_json::json;

use usecases::database_service::DatabaseService;
use usecases::prefectures;

/// 内部サーバーエラーレスポンスを生成する。
///
/// # Arguments
///
/// * `err` - エラー。
///
/// # Returns
///
/// 内部サーバーエラー。
fn internal_server_error(err: anyhow::Error) -> HttpResponse {
    HttpResponse::InternalServerError().json(json!({ "message": format!("{}", err) }))
}

/// 都道府県リストAPI。
///
/// 都道府県のリストをJSONで返却する。
///
/// # Arguments
///
/// * `db_service` - データベースサービス。
///
/// # Returns
///
/// レスポンス。
pub async fn list(db_service: web::Data<dyn DatabaseService>) -> impl Responder {
    match prefectures::list(db_service.as_ref()).await {
        Ok(prefectures) => HttpResponse::Ok().json(prefectures),
        Err(err) => internal_server_error(err),
    }
}

/// 都道府県検索API。
///
/// 指定された都道府県コードと一致する都道府県をJSONで返却する。
///
/// # Arguments
///
/// * `db_service` - データベースサービス。
/// * `path` - 引数で指定されたデータを格納するタプル。
///
/// # Returns
///
/// レスポンス。
pub async fn find_by_code(
    db_service: web::Data<dyn DatabaseService>,
    path: web::Path<(u8,)>,
) -> impl Responder {
    let code = path.into_inner().0;
    match prefectures::find_by_code(db_service.as_ref(), code).await {
        Ok(result) => match result {
            Some(prefecture) => HttpResponse::Ok().json(prefecture),
            _ => HttpResponse::NotFound().json(json!({
                "message":
                    format!(
                        "都道府県コード({})に一致する都道府県が見つかりませんでした。",
                        code
                    )
            })),
        },
        Err(err) => internal_server_error(err),
    }
}
