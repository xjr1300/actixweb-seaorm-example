use actix_web::{web, HttpResponse, Responder};
use serde_json::json;

use domains::models::accounts::AccountId;
use usecases::{
    accounts::{ChangePassword, ErrorKind, NewAccount, UpdateAccount},
    database_service::DatabaseService,
};

use crate::middlewares::JwtAuth;

/// アカウントIDを検証する。
///
/// # Arguments
///
/// * `id`: 検証する文字列。
///
/// # Returns
///
/// `Result`。返却される`Result`の内容は以下の通り。
///
/// * `Ok`: アカウントID。
/// * `Err`: BAD_REQUESTレスポンス。
fn validate_account_id(id: &str) -> Result<AccountId, HttpResponse> {
    let account_id = AccountId::try_from(id);
    if account_id.is_err() {
        return Err(HttpResponse::BadRequest().json(json!({
            "message":
                format!(
                    "URLで指定されたアカウントID({})が、ULIDの書式と異なります。",
                    id
                )
        })));
    }

    Ok(account_id.unwrap())
}

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
    let result = validate_account_id(&path.into_inner().0);
    if let Err(err) = result {
        return err;
    }
    let account_id = result.unwrap();
    // アカウントの取得を試行
    match usecases::accounts::find_by_id(db_service.as_ref(), account_id).await {
        Ok(account) => HttpResponse::Ok().json(account),
        Err(err) => {
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
    // アカウントの登録を試行
    match usecases::accounts::insert(db_service.as_ref(), new_account.into_inner()).await {
        Ok(account) => HttpResponse::Created().json(account),
        Err(err) => {
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
    path: web::Path<(String,)>,
    update_account: web::Json<UpdateAccount>,
) -> impl Responder {
    // アカウントIDを検証
    let result = validate_account_id(&path.into_inner().0);
    if let Err(err) = result {
        return err;
    }
    let account_id = result.unwrap();
    // 更新するアカウントアカウントIDを検証
    if account_id.value.to_string() != update_account.id {
        return HttpResponse::BadRequest().json(json!({
            "message":
                format!(
                    "URLで指定されたアカウントID({})とリクエストボディに指定されたアカウントID({})が異なります。",
                    account_id.value, update_account.id,
                )
        }));
    }
    // アカウントの更新を試行
    match usecases::accounts::update(db_service.as_ref(), update_account.into_inner()).await {
        Ok(account) => HttpResponse::Ok().json(account),
        Err(err) => {
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

/// アカウント削除API
///
/// URLで指定されたアカウントIDと一致するアカウントが存在しない場合は、
/// 削除に成功したと判断して`NO CONTENT`を返却する。
///
/// # Arguments
///
/// * `db_service` - データベースサービス。
/// * `path` - 削除するアカウントのアカウントIDを格納したタプル。
///
/// # Returns
///
/// レスポンス。
pub async fn delete(
    db_service: web::Data<dyn DatabaseService>,
    path: web::Path<(String,)>,
) -> impl Responder {
    // アカウントIDを検証
    let result = validate_account_id(&path.into_inner().0);
    if let Err(err) = result {
        return err;
    }
    let account_id = result.unwrap();
    // アカウントの削除を試行
    match usecases::accounts::delete(db_service.as_ref(), account_id.clone()).await {
        Ok(_) => HttpResponse::NoContent().json(json!({
            "message": format!("アカウント({})を削除しました。", account_id.value)
        })),
        Err(err) => {
            let mut response = match err.code {
                ErrorKind::InternalServerError => HttpResponse::InternalServerError(),
                _ => HttpResponse::BadRequest(),
            };
            response.json(json!({"message": err.message }))
        }
    }
}

/// パスワードを変更する。
///
/// #Arguments
///
/// * `repos` - リポジトリエクステンション。
/// * `id` - アカウントID。
/// * `data` - パスワード変更データ。
/// ```bash
/// curl --include --request POST --header "Authorization: Bearer <token>; Content-Type: application/json" \
/// --data '{"id": "<account-id>", "oldPassword": "<old-password>", "newPassword": "<new-password>"}' \
/// http://127.0.0.1:8000/accounts/change_password/<account-id>
/// ```
pub async fn change_password(
    db_service: web::Data<dyn DatabaseService>,
    path: web::Path<(String,)>,
    data: web::Json<ChangePassword>,
    jwt_auth: JwtAuth,
) -> impl Responder {
    // 認証されているか確認
    let claims;
    match jwt_auth {
        JwtAuth::Anonymous => {
            return HttpResponse::Unauthorized().json(json!({"message": "認証されていません。"}));
        }
        JwtAuth::Authenticate(c) => claims = c,
    };
    // アカウントIDを検証
    let result = validate_account_id(&path.into_inner().0);
    if let Err(err) = result {
        return err;
    }
    let account_id = result.unwrap();
    // URLで指定されたアカウントIDとJSONデータに記録されているアカウントIDが異なる場合はエラー
    let data = data.into_inner();
    if account_id.value.to_string() != data.id {
        let body = json!({
            "message": "URLで指定されたアカウントIDとリクエストボディに指定されたアカウントIDが異なります。"
        });
        return HttpResponse::BadRequest().json(json!(body));
    }
    // URLで指定されたアカウントIDとJWTトークンに指定されたアカウントIDが異なる場合はエラー
    if account_id.value.to_string() != claims.sub {
        let body = json!({
            "message": "URLで指定されたアカウントIDとJWTトークンに指定されたアカウントIDが異なります。"
        });
        return HttpResponse::BadRequest().json(json!(body));
    }
    // アカウントのパスワードの変更を試行
    match usecases::accounts::change_password(
        db_service.as_ref(),
        account_id,
        &data.old_password,
        &data.new_password,
    )
    .await
    {
        Ok(_) => HttpResponse::Ok().json(json!({"message": "パスワードを変更しました。"})),
        Err(err) => {
            let mut response = match err.code {
                ErrorKind::InvalidOldPassword => HttpResponse::BadRequest(),
                ErrorKind::InvalidNewPassword => HttpResponse::BadRequest(),
                ErrorKind::WrongPassword => HttpResponse::BadRequest(),
                _ => HttpResponse::InternalServerError(),
            };
            response.json(json!({"message": err.message}))
        }
    }
}
