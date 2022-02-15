use actix_web::{web, HttpResponse, Responder};
use serde_json::json;

use domains::models::accounts::AccountId;
use usecases::{accounts::ErrorKind, database_service::DatabaseService};

/// アカウント検索API。
///
/// 指定されたアカウントIDと一致するアカウントをJSONで返却する。
///
/// # Arguments
///
/// * `db_service` - データベースサービス。
/// * `path` - 引数で指定されたデータを格納するタプル。
///
/// # Returns
///
/// レスポンス。
pub async fn find_by_id(
    db_service: web::Data<dyn DatabaseService>,
    path: web::Path<(String,)>,
) -> impl Responder {
    // アカウントIDを検証
    let id = path.into_inner().0;
    let account_id = AccountId::try_from(id.clone());
    if account_id.is_err() {
        return HttpResponse::BadRequest().json(json!({
            "message":
                format!(
                    "URLで指定されたアカウントID({})が、ULIDの書式と異なります。",
                    id
                )
        }));
    }
    let account_id = account_id.unwrap();
    // アカウントの取得を試行
    match usecases::accounts::find_by_id(db_service.as_ref(), account_id).await {
        Ok(account) => HttpResponse::Ok().json(account),
        Err(err) => {
            log::error!("{:?}", err);
            let mut response = match err.code {
                ErrorKind::InternalServerError => HttpResponse::InternalServerError(),
                ErrorKind::NotFound => HttpResponse::NotFound(),
                _ => HttpResponse::BadRequest(),
            };
            response.json(json!({"message": err.message }))
        }
    }
}
