use std::{net::SocketAddr, sync::Arc};

use actix_web::{web::Data, App, HttpServer};
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
            .service(handlers::hello)
    })
    .bind(address)?
    .run()
    .await?;

    Ok(())
}
