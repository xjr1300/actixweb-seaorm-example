use actix_web::{App, HttpServer};
use anyhow::anyhow;

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

/// Web APIサーバーのエントリポイント
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // ロギングを設定
    init_logging().unwrap();

    HttpServer::new(|| App::new().service(hello))
        .bind("127.0.0.1:8000")?
        .run()
        .await
}
