pub mod jwt_token;

use std::{env, net::Ipv4Addr, str::FromStr};

use dotenv::dotenv;
use once_cell::sync::Lazy;

/// 環境変数
#[derive(Debug)]
pub struct EnvValues {
    /// JWTトークン秘密鍵。
    pub jwt_token_secret_key: String,
    /// JWTアクセストークン有効秒数。
    pub access_token_seconds: i64,
    /// JWTリフレッシュトークン有効秒数。
    pub refresh_token_seconds: i64,
    /// WebサーバーのIPアドレス。
    pub web_server_address: Ipv4Addr,
    /// Webサーバーのポート番号。
    pub web_server_port: u16,
    /// ログレベル。
    pub log_level: String,
    /// log4rs設定ファイル。
    pub log4rs_config: String,
    /// パスワードハッシュ化関数。
    pub password_hash_func: String,
    /// パスワードソルト文字数。
    pub password_sault_len: usize,
    /// パスワードペッパー。
    pub password_pepper: String,
    /// パスワードハッシュ化ラウンド数。
    pub password_hash_round: u32,
    /// データベースURL。
    pub database_url: String,
}

/// 環境変数
pub static ENV_VALUES: Lazy<EnvValues> = Lazy::new(|| {
    dotenv().ok();

    let web_server_address =
        env::var("WEB_SERVER_ADDRESS").expect("環境変数にWEB_SERVER_ADDRESSが設定されていません。");
    let web_server_address = Ipv4Addr::from_str(&web_server_address)
        .expect("環境変数に設定してあるWEB_SERVE_ADDRESSが不正です。");

    EnvValues {
        jwt_token_secret_key: env::var("JWT_TOKEN_SECRET_KEY")
            .expect("環境変数にSECRET_KEYが設定されていません。"),
        access_token_seconds: env::var("ACCESS_TOKEN_SECONDS")
            .expect("環境変数にACCESS_TOKEN_SECONDSが設定されていません。")
            .parse::<i64>()
            .expect("環境変数に設定されているACCESS_TOKEN_SECONDSが不正です。"),
        refresh_token_seconds: env::var("REFRESH_TOKEN_SECONDS")
            .expect("環境変数にREFRESH_TOKEN_SECONDSが設定されていません。")
            .parse::<i64>()
            .expect("環境変数に設定されているREFRESH_TOKEN_SECONDSが不正です。"),
        web_server_address,
        web_server_port: env::var("WEB_SERVER_PORT")
            .expect("環境変数にWEB_SERVER_PORTが設定されていません。")
            .parse::<u16>()
            .expect("環境変数に設定されているWEB_SERVER_PORTが不正です。"),
        log_level: env::var("RUST_LOG").expect("環境変数にRUST_LOGが設定されていません。"),
        log4rs_config: env::var("LOG4RS_CONFIG")
            .expect("環境変数にLOG4RS_CONFIGが設定されていません。"),
        password_hash_func: env::var("PASSWORD_HASH_FUNC")
            .expect("環境変数にPASSWORD_HASH_FUNCが設定されていません。"),
        password_sault_len: env::var("PASSWORD_SAULT_LEN")
            .expect("環境変数にPASSWORD_SAULT_LENが設定されていません。")
            .parse::<usize>()
            .expect("環境変数に設定されているPASSWORD_SAULT_LENが不正です。"),
        password_pepper: env::var("PASSWORD_PEPPER")
            .expect("環境変数にPASSWORD_PEPPERが設定されていません。"),
        password_hash_round: env::var("PASSWORD_HASH_ROUND")
            .expect("環境変数にPASSWORD_HASH_ROUNDが設定されていません。")
            .parse::<u32>()
            .expect("環境変数に設定されているPASSWORD_HASH_ROUNDが不正です。"),
        database_url: env::var("DATABASE_URL")
            .expect("環境変数にDATABASE_URLが設定されていません。"),
    }
});
