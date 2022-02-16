use std::borrow::Cow;

use chrono::{DateTime, Duration, FixedOffset};
use sea_orm::{ConnectionTrait, DatabaseConnection, DatabaseTransaction};
use serde::{Deserialize, Serialize};

use common::{
    jwt_token::{gen_jwt_token, Claims},
    ENV_VALUES,
};
use domains::{
    models::{
        accounts::{Account, AccountId, RawPassword},
        auth::{JwtToken, JwtTokenWithExpiredAt, JwtTokens, JwtTokensId},
        common::{local_now, EmailAddress},
    },
    repositories::{accounts::AccountRepository, auth::JwtTokensRepository},
    services::auth::authenticate,
};

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
    let mut claims = Claims {
        sub: account_id.value.to_string(),
        exp: access_expired_at.timestamp(),
    };
    let access = gen_jwt_token(&claims);
    if let Err(err) = access {
        return Err(internal_server_error(err.into()));
    }
    claims.exp = refresh_expired_at.timestamp();
    let refresh = gen_jwt_token(&claims);
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
