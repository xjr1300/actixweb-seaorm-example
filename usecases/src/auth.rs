use std::{borrow::Cow, collections::BTreeMap};

use chrono::{DateTime, Duration, FixedOffset};
use common::ENV_VALUES;
use domains::{
    models::{
        accounts::{Account, AccountId, RawPassword},
        auth::{JwtToken, JwtTokenWithExpiredAt, JwtTokens, JwtTokensId},
        common::{local_now, EmailAddress},
    },
    repositories::{accounts::AccountRepository, auth::JwtTokensRepository},
    services::auth::authenticate,
};
use hmac::{Hmac, Mac};
use jwt::{SignWithKey, VerifyWithKey};
use sea_orm::{ConnectionTrait, DatabaseConnection, DatabaseTransaction};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use crate::database_service::DatabaseService;

/// 認証ユースケースエラー区分
#[derive(Debug, Clone)]
pub enum ErrorKind {
    /// サーバー内部エラー
    InternalServerError,
    /// アカウントに登録したEメールアドレス、またはパスワードが異なる。
    InvalidCredential,
    /// Eメールアドレスが不正
    InvalidEmailAddress,
    /// パスワードが不正
    InvalidPassword,
}

/// 認証ユースケースエラー
#[derive(Debug, Clone)]
pub struct Error {
    // エラー区分コード。
    pub code: ErrorKind,
    /// エラーメッセージ。
    pub message: Cow<'static, str>,
}

/// クレデンシャル
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Credential {
    /// Eメールアドレス。
    pub email: String,
    /// パスワード。
    pub password: String,
}

/// 有効期限付きアクセス・リフレッシュトークンデータトランスファーオブジェクト
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JwtTokensDto {
    /// トークンID。
    pub id: String,
    /// アカウントID。
    pub account_id: String,
    /// アクセストークン。
    pub access: String,
    /// アクセストークン有効期限。
    pub access_expired_at: DateTime<FixedOffset>,
    /// リフレッシュトークン。
    pub refresh: String,
    /// リフレッシュトークン有効期限。
    pub refresh_expired_at: DateTime<FixedOffset>,
}

fn to_email(value: &str) -> Result<EmailAddress, Error> {
    match EmailAddress::new(value) {
        Ok(value) => Ok(value),
        Err(e) => Err(Error {
            code: ErrorKind::InvalidEmailAddress,
            message: format!("{}", e).into(),
        }),
    }
}

fn to_raw_password(value: &str) -> Result<RawPassword, Error> {
    match RawPassword::new(value) {
        Ok(value) => Ok(value),
        Err(err) => Err(Error {
            code: ErrorKind::InvalidPassword,
            message: format!("{}", err).into(),
        }),
    }
}

/// インターナルサーバーエラーを生成する。
///
/// # Arguments
///
/// * `err` - エラー。
///
/// # Returns
///
/// インターナルエラー。
fn internal_server_error(err: Box<dyn std::error::Error>) -> Error {
    Error {
        code: ErrorKind::InternalServerError,
        message: format!("{}", err).into(),
    }
}

/// トランザクションを開始する。
///
/// # Arguments
///
/// * `conn` - データベースコネクション。
///
/// # Returns
///
/// `Result`。返却される`Result`の内容は以下の通り。
///
/// * `Ok`: データベーストランザクション。
/// * `Err`: エラー。
async fn begin_transaction(conn: &DatabaseConnection) -> Result<DatabaseTransaction, Error> {
    let txn = conn.begin().await;
    if let Err(err) = txn {
        return Err(internal_server_error(Box::new(err)));
    }

    Ok(txn.unwrap())
}

/// アカウントを認証する。
///
/// # Arguments
///
/// * `repos` - リポジトリエクステンション。
/// * `txn` - データベーストランザクション。
/// * `email` - 認証するアカウントのEメールアドレス。
/// * `password` - 認証するアカウントのパスワード。
///
/// # Returns
///
/// `Result`。返却される`Result`の内容は以下の通り。
///
/// * `Ok`: 認証に成功したアカウント。
/// * `Err`: エラー。`
async fn authenticate_account(
    repo: &dyn AccountRepository,
    email: EmailAddress,
    password: RawPassword,
) -> Result<Account, Error> {
    let result = authenticate(repo, email, password).await;
    if let Err(err) = result {
        return Err(internal_server_error(err.into()));
    }
    let account = result.unwrap();
    if account.is_none() {
        return Err(Error {
            code: ErrorKind::InvalidCredential,
            message: "アカウントで使用しているEメールアドレス、またはパスワードが間違っています。"
                .into(),
        });
    }

    Ok(account.unwrap())
}

/// JWTトークンを生成する。
///
/// # Arguments
///
/// * `id` - アカウントID。
///
/// # Returns
///
/// `Result`。返却される`Result`の内容は以下の通り。
///
/// * `Ok`: JWT。
/// * `Err`: エラー。
pub fn gen_jwt_token(id: AccountId, expired: DateTime<FixedOffset>) -> anyhow::Result<String> {
    // 環境変数から秘密鍵を取得
    let secret_key = &ENV_VALUES.jwt_token_secret_key;
    // アカウントIDを文字列に変更
    let id = id.value.to_string();
    // 有効期限をUnixエポック(1970-01-01(UTC))からの経過秒数を示す文字列に変更
    let exp = expired.timestamp().to_string();
    // 鍵を生成
    let key: Hmac<Sha256> = Hmac::new_from_slice(secret_key.as_bytes())?;
    // JWTを生成
    let mut claims = BTreeMap::new();
    claims.insert("sub", &id);
    claims.insert("exp", &exp);
    let token = claims.sign_with_key(&key)?;

    Ok(token)
}

#[cfg(test)]
mod auth_tests {
    use super::*;
    use common::ENV_VALUES;
    use domains::models::accounts::AccountId;
    use dotenv;
    use jwt::VerifyWithKey;

    /// JWTを正常に生成できることを確認する。
    #[test]
    fn test_gen_jwt() {
        dotenv::from_filename(".env.dev").ok();
        // JWTを生成
        let id = AccountId::gen();
        let expired = local_now(None) + Duration::days(1);
        let token = gen_jwt_token(id.clone(), expired);
        if let Err(ref err) = token {
            assert!(false, "JWTを生成できませんでした。{:?}。", err);
        }
        // 生成したトークンを検証
        let token = token.unwrap();
        let secret_key = &ENV_VALUES.jwt_token_secret_key;
        let key: Hmac<Sha256> = Hmac::new_from_slice(secret_key.as_bytes()).unwrap();
        let claims: BTreeMap<String, String> = token.verify_with_key(&key).unwrap();
        assert_eq!(claims["sub"], id.value.to_string());
        assert_eq!(claims["exp"], expired.timestamp().to_string());
    }
}

/// 有効期限付きアクセス・リフレッシュトークンを生成する。
///
/// # Arguments
///
/// * `account_id` - アカウントID。
///
/// # Returns
///
/// `Result`。返却される`Result`の内容は以下の通り。
///
/// * `Ok`: 有効期限付きアクセス・リフレッシュトークン。
/// * `Err`: エラー。
fn gen_jwt_tokens(account_id: AccountId) -> Result<JwtTokens, Error> {
    // 有効期限を設定
    let now = local_now(None);
    let access_expired_at = now + Duration::seconds(ENV_VALUES.access_token_seconds);
    let refresh_expired_at = now + Duration::seconds(ENV_VALUES.refresh_token_seconds);
    // トークンを生成
    let access = gen_jwt_token(account_id.clone(), access_expired_at);
    if let Err(err) = access {
        return Err(internal_server_error(err.into()));
    }
    let refresh = gen_jwt_token(account_id.clone(), refresh_expired_at);
    if let Err(err) = refresh {
        return Err(internal_server_error(err.into()));
    }
    // アクセストークンとリフレッシュトークンを生成
    let access = JwtTokenWithExpiredAt {
        token: JwtToken::new(&access.unwrap()).unwrap(),
        expired_at: access_expired_at,
    };
    let refresh = JwtTokenWithExpiredAt {
        token: JwtToken::new(&refresh.unwrap()).unwrap(),
        expired_at: refresh_expired_at,
    };

    Ok(JwtTokens::new(
        JwtTokensId::gen(),
        account_id,
        access,
        refresh,
    ))
}

/// 有効期限付きアクセス・リフレッシュトークンをデータベースに保存する。
///
/// # Arguments
///
/// * `repos` - リポジトリエクステンション。
/// * `txn` - データベーストランザクション。
/// * `tokens` - 有効期限付きアクセス・リフレッシュトークン。
///
/// # Returns
///
/// `Result`。返却される`Result`の内容は以下の通り。
///
/// * `Ok`: 有効期限付きアクセス・リフレッシュトークン。
/// * `Err`: エラー。
async fn save_jwt_tokens(
    repo: &dyn JwtTokensRepository,
    tokens: &JwtTokens,
) -> Result<JwtTokens, Error> {
    match repo.insert(tokens).await {
        Ok(result) => Ok(result),
        Err(err) => Err(internal_server_error(err.into())),
    }
}

/// 有効期限付きアクセス・リフレッシュトークンを生成して返却する。
///
/// # Arguments
///
/// * `db_service` - リポジトリエクステンション。
/// * `credential` - アカウントクレデンシャル。
///
/// # Returns
///
/// `Result`。返却される`Result`の内容は以下の通り。
///
/// * `Ok`: 有効期限付きアクセス・リフレッシュトークン。
/// * `Err`: エラー。
pub async fn obtain_tokens(
    db_service: &dyn DatabaseService,
    credential: Credential,
) -> Result<JwtTokensDto, Error> {
    let tokens;
    let email = to_email(&credential.email)?;
    let password = to_raw_password(&credential.password)?;

    // トランザクションを開始
    let txn = begin_transaction(&db_service.connection()).await?;
    {
        let account_repo = db_service.account(&txn);
        let jwt_repo = db_service.jwt_tokens(&txn);
        // アカウントを認証
        let account = authenticate_account(&*account_repo, email, password).await?;
        // トークンを生成
        let result = gen_jwt_tokens(account.id())?;
        // トークンを保存
        tokens = save_jwt_tokens(&*jwt_repo, &result).await?;
    }
    // トランザクションをコミット
    match txn.commit().await {
        Ok(_) => Ok(JwtTokensDto {
            id: tokens.id().value.to_string(),
            account_id: tokens.account_id().value.to_string(),
            access: tokens.access().token.value(),
            access_expired_at: tokens.access().expired_at,
            refresh: tokens.refresh().token.value(),
            refresh_expired_at: tokens.refresh().expired_at,
        }),
        Err(err) => Err(internal_server_error(err.into())),
    }
}
