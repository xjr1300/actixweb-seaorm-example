use std::{net::SocketAddr, sync::Arc};

use actix_web::{
    self,
    web::{self, Data},
    App, HttpServer,
};
use anyhow::anyhow;
use sea_orm::Database;

use common::ENV_VALUES;
use usecases::database_service::DatabaseService;

mod database_service;
mod handlers;
use crate::database_service::DatabaseServiceImpl;

/// Web APIサーバーを起動する。
///
/// # Arguments
///
/// * `address` - Web APIサーバーのソケットアドレス。
///
/// # Returns
///
/// `Result`。返却される`Result`の内容は下記の通り。
///
/// * `Ok`: ()
/// * `Err`: エラー。
pub async fn run(address: &SocketAddr) -> anyhow::Result<()> {
    // データベースに接続
    log::info!("Connecting to database...");
    let conn = Database::connect(&ENV_VALUES.database_url)
        .await
        .map_err(|_| {
            anyhow!("環境変数に設定されているDATABASE_URLで、データベースに接続できません。")
        });
    let conn = conn.unwrap();
    log::info!("Connected to database...");
    // データベースサービスを構築
    let db_service: Arc<dyn DatabaseService> = Arc::new(DatabaseServiceImpl { conn });
    let db_service: Data<dyn DatabaseService> = Data::from(db_service);
    // Web APIサーバーを起動
    HttpServer::new(move || {
        App::new()
            .app_data(db_service.clone())
            .service(
                web::scope("/").service(web::resource("").route(web::get().to(handlers::hello))),
            )
            .service(prefecture_scope())
            .service(accounts_scope())
    })
    .bind(address)?
    .run()
    .await?;

    Ok(())
}

/// 都道府県スコープ
fn prefecture_scope() -> actix_web::Scope {
    web::scope("/prefectures")
        .service(web::resource("").route(web::get().to(handlers::prefectures::list)))
        .service(web::resource("/{code}").route(web::get().to(handlers::prefectures::find_by_code)))
}

/// アカウントスコープ
///
/// # アカウント登録API
///
/// ```bash
/// curl --include --request POST --header "Content-Type: application/json" --data '{"email": "foo@example.com", "name": "foo", "password": "012abcEFG=+", "isActive": true, "fixedNumber": "012-345-6789", "mobileNumber": "090-1234-5678", "postalCode": "012-3456", "prefectureCode": 13, "addressDetails": "千代田区永田町1-7-1"}' http://127.0.0.1:8000/accounts
/// ```
fn accounts_scope() -> actix_web::Scope {
    web::scope("/accounts")
        .service(web::resource("").route(web::post().to(handlers::accounts::insert)))
        .service(web::resource("/{id}").route(web::get().to(handlers::accounts::find_by_id)))
}
