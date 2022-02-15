use std::net::SocketAddr;

use actix_web::{App, HttpServer};
use anyhow::anyhow;
use sea_orm::Database;

use common::ENV_VALUES;

mod handlers;

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
    let _conn = Database::connect(&ENV_VALUES.database_url)
        .await
        .map_err(|_| {
            anyhow!("環境変数に設定されているDATABASE_URLで、データベースに接続できません。")
        });
    log::info!("Connected to database...");

    HttpServer::new(|| App::new().service(handlers::hello))
        .bind(address)?
        .run()
        .await?;

    Ok(())
}
