use actix_web::{web, HttpResponse, Responder};
use serde_json::json;

use domains::models::accounts::AccountId;
use usecases::{
    accounts::{ErrorKind, NewAccount, UpdateAccount},
    database_service::DatabaseService,
};

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

/// アカウント登録API
///
/// # Arguments
///
/// * `db_service` - データベースサービス。
/// * `new_account` - 登録するアカウント。
///
/// # Returns
///
/// レスポンス。
pub async fn insert(
    db_service: web::Data<dyn DatabaseService>,
    new_account: web::Json<NewAccount>,
) -> impl Responder {
    match usecases::accounts::insert(db_service.as_ref(), new_account.into_inner()).await {
        Ok(account) => HttpResponse::Created().json(account),
        Err(err) => {
            log::error!("{:?}", err);
            let mut response = match err.code {
                ErrorKind::InternalServerError => HttpResponse::InternalServerError(),
                ErrorKind::PrefectureNotFound => HttpResponse::NotFound(),
                _ => HttpResponse::BadRequest(),
            };
            response.json(json!({"message": err.message}))
        }
    }
}

/// アカウント更新API
///
/// # Arguments
///
/// * `db_service` - データベースサービス。
/// * `update_account` - 更新するアカウント。
///
/// # Returns
///
/// レスポンス。
pub async fn update(
    db_service: web::Data<dyn DatabaseService>,
    update_account: web::Json<UpdateAccount>,
) -> impl Responder {
    match usecases::accounts::update(db_service.as_ref(), update_account.into_inner()).await {
        Ok(account) => HttpResponse::Ok().json(account),
        Err(err) => {
            log::error!("{:?}", err);
            let mut response = match err.code {
                ErrorKind::InternalServerError => HttpResponse::InternalServerError(),
                ErrorKind::NotFound => HttpResponse::NotFound(),
                ErrorKind::PrefectureNotFound => HttpResponse::NotFound(),
                _ => HttpResponse::BadRequest(),
            };
            response.json(json!({"message": err.message}))
        }
    }
}
