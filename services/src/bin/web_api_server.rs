use std::net::{IpAddr, SocketAddr};

use actix_web::{App, HttpServer};
use anyhow::anyhow;
use sea_orm::{Database, DatabaseConnection};

use adapters::handlers::hello;
use common::ENV_VALUES;

/// ログの出力方法を設定する。
///
/// # Returns
///
/// `Result`。返却される`Result`の内容は以下の通り。
///
/// * `Ok`: ()。
/// * `Err`: エラー内容。
fn init_logging() -> anyhow::Result<()> {
    // ロギング設定ファイルを開く。
    match log4rs::init_file(&ENV_VALUES.log4rs_config, Default::default()) {
        Ok(_) => Ok(()),
        Err(err) => Err(anyhow!(
            "ファイル({})からロギング設定を得られません。{:?}",
            ENV_VALUES.log4rs_config,
            err,
        )),
    }
}

/// 環境変数からホスト名とポート番号を取得して、Webアプリケーションのソケットアドレスを返却する。
///
/// # Returns
///
/// `Result`。返却される`Result`の内容を以下に示す。
///
/// * `Ok`: ソケットアドレス。
/// * `Err`: エラー。
fn server_socket_address() -> anyhow::Result<SocketAddr> {
    Ok(SocketAddr::new(
        IpAddr::V4(ENV_VALUES.web_server_address),
        ENV_VALUES.web_server_port,
    ))
}

/// データーベースコネクションを取得する。
///
/// # Returns
///
/// `Result`。返却される`Result`の内容を以下に示す。
///
/// * `Ok`: データベースコネクション。
/// * `Err`: エラー。
async fn database_connection() -> anyhow::Result<DatabaseConnection> {
    Database::connect(&ENV_VALUES.database_url)
        .await
        .map_err(|_| {
            anyhow!("環境変数に設定されているDATABASE_URLで、データベースに接続できません。")
        })
}

/// Web APIサーバーのエントリポイント
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 環境変数の内容でロギングを設定
    init_logging().unwrap();

    // 環境変数からWeb APIサーバーのソケットアドレスを取得
    let address = server_socket_address().unwrap();

    // データベースに接続
    log::info!("Connecting to database...");
    let _conn = database_connection().await.unwrap();
    log::info!("Connected to database...");

    // Web APIサーバーを起動
    HttpServer::new(|| App::new().service(hello))
        .bind(address)?
        .run()
        .await
}
