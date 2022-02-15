use std::{borrow::Cow, sync::Arc};

use chrono::{DateTime, FixedOffset};
use sea_orm::{ConnectionTrait, DatabaseConnection, DatabaseTransaction};
use serde::{Deserialize, Serialize};

use domains::{
    models::{
        accounts::{
            optional_phone_number, optional_phone_number_string, Account, AccountId, AccountName,
            FixedMobileNumbers, HashedPassword, RawPassword,
        },
        common::{
            local_now, Address, AddressDetails, EmailAddress, PhoneNumber, PostalCode, Prefecture,
        },
    },
    services::auth::verify_password,
};

use crate::database_service::DatabaseService;

/// アカウントユースケースエラー区分
#[derive(Debug, Clone)]
pub enum ErrorKind {
    /// サーバー内部エラー
    InternalServerError,
    /// アカウントが見つからない
    NotFound,
    /// 都道府県が見つからない
    PrefectureNotFound,
    /// アカウントIDが不正
    InvalidAccountId,
    /// Eメールアドレスが不正
    InvalidEmailAddress,
    /// アカウント名が不正
    InvalidName,
    /// パスワードが不正
    InvalidPassword,
    /// パスワードが間違っている
    WrongPassword,
    /// 固定電話番号が不正
    InvalidFixedNumber,
    /// 携帯電話番号が不正
    InvalidMobileNumber,
    /// 固定携帯電話番号が不正
    InvalidPhoneNumbers,
    /// 郵便番号が不正
    InvalidPostalCode,
    /// 市区町村以下住所が不正
    InvalidAddressDetails,
    /// 古いパスワードが不正
    InvalidOldPassword,
    /// 新しいパスワードが不正
    InvalidNewPassword,
}

/// アカウントユースケースエラー
#[derive(Debug, Clone)]
pub struct Error {
    /// エラー区分コード。
    pub code: ErrorKind,
    /// エラーメッセージ。
    pub message: Cow<'static, str>,
}

/// アカウントデータトランスファーオブジェクト
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountDto {
    /// アカウントID。
    pub id: String,
    /// Eメールアドレス。
    pub email: String,
    /// アカウント名。
    pub name: String,
    /// アクティブフラグ。
    pub is_active: bool,
    /// 固定電話番号。
    pub fixed_number: Option<String>,
    /// 携帯電話番号。
    pub mobile_number: Option<String>,
    /// 郵便番号。
    pub postal_code: String,
    /// 都道府県コード。
    pub prefecture_code: u8,
    /// 市区町村以下住所。
    pub address_details: String,
    /// 最終ログイン日時。
    pub logged_in_at: Option<DateTime<FixedOffset>>,
    /// 登録日時。
    pub created_at: DateTime<FixedOffset>,
    /// 更新日時。
    pub updated_at: DateTime<FixedOffset>,
}

#[allow(clippy::from_over_into)]
impl Into<AccountDto> for Account {
    fn into(self) -> AccountDto {
        AccountDto {
            id: self.id().value.to_string(),
            email: self.email().value(),
            name: self.name().value(),
            is_active: self.is_active(),
            fixed_number: optional_phone_number_string(self.phone_numbers().fixed()),
            mobile_number: optional_phone_number_string(self.phone_numbers().mobile()),
            postal_code: self.postal_code().value(),
            prefecture_code: self.address().prefecture().code(),
            address_details: self.address().details().value(),
            logged_in_at: self.logged_in_at(),
            created_at: self.created_at(),
            updated_at: self.updated_at(),
        }
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
        return Err(internal_error(Box::new(err)));
    }

    Ok(txn.unwrap())
}

/// 都道府県を取得する。
///
/// # Arguments
///
/// * `repos`: リポジトリエクステンション。
/// * `txn`: データベーストランザクション。
/// * `code`: 都道府県コード。
///
/// # Returns
///
/// * `Ok`: 都道府県。
/// * `Err`: エラー。
async fn retrieve_prefecture(
    repos: &dyn DatabaseService,
    txn: &DatabaseTransaction,
    code: u8,
) -> Result<Prefecture, Error> {
    let repo = repos.prefecture(txn);
    let result = repo.find_by_code(code).await;
    if let Err(err) = result {
        return Err(internal_error(err.into()));
    }
    let result = result.unwrap();
    // 都道府県を取得できたか確認
    if result.is_none() {
        return Err(usecase_error(
            ErrorKind::PrefectureNotFound,
            format!(
                "アカウントに記録されていた都道府県コード({})と一致する都道府県が見つかりません。",
                code
            )
            .into(),
        ));
    }

    Ok(result.unwrap())
}

/// 内部サーバーエラーを生成する。
///
/// # Arguments
///
/// * `err` - エラー。
///
/// # Returns
///
/// 内部サーバーエラー。
fn internal_error(err: Box<dyn std::error::Error>) -> Error {
    Error {
        code: ErrorKind::InternalServerError,
        message: format!("{}", err).into(),
    }
}

/// ユースケースエラーを生成する。
///
/// # Arguments
///
/// * `code`: エラーの種類。
/// * `message`: エラーメッセージ。
///
/// # Returns
///
/// ユースケースエラー。
fn usecase_error(code: ErrorKind, message: Cow<'static, str>) -> Error {
    Error { code, message }
}

/// アカウントを検索する。
///
/// # Arguments
///
/// * `repos` - リポジトリエクステンション。
/// * `txn` - データベーストランザクション。
/// * `id` - アカウントID。
///
/// # Returns
///
/// `Result`。返却される`Result`の内容は以下の通り。
///
/// * `Ok`: アカウント。
/// * `Err`: エラー。
async fn find_account(
    repos: &dyn DatabaseService,
    txn: &DatabaseTransaction,
    id: AccountId,
) -> Result<Account, Error> {
    // アカウントを検索
    let result = repos.account(txn).find_by_id(id.clone()).await;
    if let Err(err) = result {
        return Err(internal_error(err.into()));
    }
    let result = result.unwrap();
    // アカウントが見つからなかった場合
    if result.is_none() {
        return Err(usecase_error(
            ErrorKind::NotFound,
            format!(
                "アカウントID({})と一致するアカウントが見つかりません。",
                id.value.to_string()
            )
            .into(),
        ));
    }

    Ok(result.unwrap())
}

/// 指定されたアカウントIDと一致するアカウントを返却する。
///
/// # Arguments
///
/// * `repos` - リポジトリエクステンション。
/// * `id` - アカウントID。
///
/// # Returns
///
/// `Result`。返却される`Result`の内容は以下の通り。
///
/// * `Ok`: アカウント。検索できなかった場合は`None`。
/// * `Err`: エラー。
pub async fn find_by_id(
    repos: Arc<dyn DatabaseService>,
    id: AccountId,
) -> Result<AccountDto, Error> {
    // トランザクションを開始
    let txn = begin_transaction(&repos.connection()).await?;
    // アカウントを取得
    let account = find_account(&*repos, &txn, id.clone()).await?;
    // トランザクションをコミット
    match txn.commit().await {
        Ok(_) => Ok(account.into()),
        Err(err) => Err(internal_error(err.into())),
    }
}

fn to_account_id(value: String) -> Result<AccountId, Error> {
    match AccountId::try_from(value) {
        Ok(value) => Ok(value),
        Err(e) => Err(usecase_error(
            ErrorKind::InvalidAccountId,
            format!("{}", e).into(),
        )),
    }
}

fn to_email(value: &str) -> Result<EmailAddress, Error> {
    match EmailAddress::new(value) {
        Ok(value) => Ok(value),
        Err(e) => Err(usecase_error(
            ErrorKind::InvalidEmailAddress,
            format!("{}", e).into(),
        )),
    }
}

fn to_name(value: &str) -> Result<AccountName, Error> {
    match AccountName::new(value) {
        Ok(value) => Ok(value),
        Err(err) => Err(usecase_error(
            ErrorKind::InvalidName,
            format!("{}", err).into(),
        )),
    }
}

fn to_raw_password(value: &str) -> Result<RawPassword, Error> {
    match RawPassword::new(value) {
        Ok(value) => Ok(value),
        Err(err) => Err(usecase_error(
            ErrorKind::InvalidPassword,
            format!("{}", err).into(),
        )),
    }
}

fn to_phone_number(value: Option<&str>, prefix: &str) -> Result<Option<PhoneNumber>, Error> {
    match optional_phone_number(value) {
        Ok(value) => Ok(value),
        Err(err) => {
            let (code, name) = if prefix == "fixed" {
                (ErrorKind::InvalidFixedNumber, "固定")
            } else {
                (ErrorKind::InvalidMobileNumber, "携帯")
            };
            Err(usecase_error(code, format!("{}{}", name, err).into()))
        }
    }
}

fn to_phone_numbers(
    fixed: Option<PhoneNumber>,
    mobile: Option<PhoneNumber>,
) -> Result<FixedMobileNumbers, Error> {
    match FixedMobileNumbers::new(fixed, mobile) {
        Ok(value) => Ok(value),
        Err(err) => Err(usecase_error(
            ErrorKind::InvalidPhoneNumbers,
            format!("{}", err).into(),
        )),
    }
}

fn to_postal_code(value: &str) -> Result<PostalCode, Error> {
    match PostalCode::new(value) {
        Ok(value) => Ok(value),
        Err(err) => Err(usecase_error(
            ErrorKind::InvalidPostalCode,
            format!("{}", err).into(),
        )),
    }
}

fn to_address_details(value: &str) -> Result<AddressDetails, Error> {
    match AddressDetails::new(value) {
        Ok(value) => Ok(value),
        Err(err) => Err(usecase_error(
            ErrorKind::InvalidAddressDetails,
            format!("{}", err).into(),
        )),
    }
}

/// 新規アカウント
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewAccount {
    /// Eメールアドレス。
    pub email: String,
    /// アカウント名。
    pub name: String,
    /// パスワード。
    pub password: String,
    /// アクティブフラグ。
    pub is_active: bool,
    /// 固定電話番号。
    pub fixed_number: Option<String>,
    /// 携帯電話番号。
    pub mobile_number: Option<String>,
    /// 郵便番号。
    pub postal_code: String,
    /// 都道府県コード。
    pub prefecture_code: u8,
    /// 市区町村以下住所。
    pub address_details: String,
}

/// アカウントを登録する。
///
/// # Arguments
///
/// * `repos` - アカウントリポジトリ。
/// * `new` - 登録するアカウント。
///
/// # Returns
///
/// `Result`。返却される`Result`の内容は以下の通り。
///
/// * `Ok`: 登録したアカウント。
/// * `Err`: エラー。
pub async fn insert(repos: Arc<dyn DatabaseService>, new: NewAccount) -> Result<AccountDto, Error> {
    // 返却するアカウント
    let new_account: Account;
    // アカウントに設定する値を生成
    let email = to_email(&new.email)?;
    let name = to_name(&new.name)?;
    let raw_password = to_raw_password(&new.password)?;
    let fixed_number = to_phone_number(new.fixed_number.as_deref(), "fixed")?;
    let mobile_number = to_phone_number(new.mobile_number.as_deref(), "mobile")?;
    let phone_numbers = to_phone_numbers(fixed_number, mobile_number)?;
    let postal_code = to_postal_code(&new.postal_code)?;
    let address_details = to_address_details(&new.address_details)?;
    // トランザクションを開始
    let txn = begin_transaction(&repos.connection()).await?;
    {
        // アカウントに記録されていた都道府県コードから都道府県を取得
        let prefecture = retrieve_prefecture(&*repos, &txn, new.prefecture_code).await?;
        // 登録するアカウントを生成
        let account = Account::new(
            email,
            name,
            raw_password,
            new.is_active,
            phone_numbers,
            postal_code,
            Address::new(prefecture, address_details),
        );
        // アカウントを登録
        let account_repo = repos.account(&txn);
        let result = account_repo.insert(&account).await;
        if let Err(err) = result {
            return Err(internal_error(err.into()));
        }
        new_account = result.unwrap();
    }
    // トランザクションをコミット
    match txn.commit().await {
        Ok(_) => Ok(new_account.into()),
        Err(err) => Err(internal_error(err.into())),
    }
}

/// 更新アカウント
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAccount {
    /// アカウントID。
    pub id: String,
    /// アカウント名。
    pub name: String,
    /// アクティブフラグ。
    pub is_active: bool,
    /// 固定電話番号。
    pub fixed_number: Option<String>,
    /// 携帯電話番号。
    pub mobile_number: Option<String>,
    /// 郵便番号。
    pub postal_code: String,
    /// 都道府県コード。
    pub prefecture_code: u8,
    /// 市区町村以下住所。
    pub address_details: String,
}

/// アカウントを更新する。
///
/// # Arguments
///
/// * `repos`: リポジトリエクステンション。
/// * `account`: 更新するアカウント。
///
/// # Returns
///
/// `Result`。返却される`Result`の内容は以下の通り。
///
/// * `Ok`: 更新後のアカウント。アカウントが見つからなかった場合、都道府県コードが不正な場合はNone。
/// * `Err`: エラー。
pub async fn update(
    repos: Arc<dyn DatabaseService>,
    account: UpdateAccount,
) -> Result<AccountDto, Error> {
    // 返却するアカウント
    let updated_account: Account;
    // アカウントIDを生成
    let account_id = to_account_id(account.id)?;
    // 更新する値を生成
    let name = to_name(&account.name)?;
    let fixed_number = to_phone_number(account.fixed_number.as_deref(), "fixed")?;
    let mobile_number = to_phone_number(account.mobile_number.as_deref(), "mobile")?;
    let phone_numbers = to_phone_numbers(fixed_number, mobile_number)?;
    let postal_code = to_postal_code(&account.postal_code)?;
    let address_details = to_address_details(&account.address_details)?;
    // トランザクションを開始
    let txn = begin_transaction(&repos.connection()).await?;
    {
        // アカウントに記録されていた都道府県コードから都道府県を取得
        let prefecture = retrieve_prefecture(&*repos, &txn, account.prefecture_code).await?;
        // 更新するアカウントを取得
        let mut target = find_account(&*repos, &txn, account_id).await?;
        // 更新するアカウントに値を設定
        target.set_name(name);
        target.set_is_active(account.is_active);
        target.set_phone_numbers(phone_numbers);
        target.set_postal_code(postal_code);
        target.set_address(Address::new(prefecture, address_details));
        target.set_updated_at(local_now(None));
        // アカウントを更新
        let result = repos.account(&txn).update(&target).await;
        if let Err(err) = result {
            return Err(internal_error(err.into()));
        }
        updated_account = result.unwrap();
    }
    // トランザクションをコミット
    match txn.commit().await {
        Ok(_) => Ok(updated_account.into()),
        Err(err) => Err(internal_error(err.into())),
    }
}

/// アカウントを削除する。
///
/// # Arguments
///
/// * `repos` - アカウントリポジトリ。
/// * `id` - 削除するアカウントのID。
///
/// # Returns
///
/// `Result`。返却される`Result`の内容は以下の通り。
///
/// * `Ok`: 削除したアカウント。
/// * `Err`: エラー。
pub async fn delete(repos: Arc<dyn DatabaseService>, id: AccountId) -> Result<(), Error> {
    // トランザクションを開始
    let txn = begin_transaction(&repos.connection()).await?;
    {
        // アカウントを取得
        let _ = find_account(&*repos, &txn, id.clone()).await?;
        // アカウントを削除
        let result = repos.account(&txn).delete(id).await;
        if let Err(err) = result {
            return Err(internal_error(err.into()));
        }
    }
    // トランザクションをコミット
    match txn.commit().await {
        Ok(_) => Ok(()),
        Err(err) => Err(internal_error(err.into())),
    }
}

/// パスワード変更
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangePassword {
    /// アカウントID。
    pub id: String,
    /// 古いパスワード。
    pub old_password: String,
    /// 新しいパスワード。
    pub new_password: String,
}

/// パスワードを変更する。
///
/// # Arguments
///
/// * `repos` - リポジトリエクステンション。
/// * `id` - パスワードを変更するアカウントのアカウントID。
/// * `old_password` - 変更前のパスワード。
/// * `new_password` - 変更後のパスワード。
///
/// # Returns
///
/// `Result`。返却される`Result`の内容は以下の通り。
///
/// * `Ok`: パスワードの変更に成功した場合は`()`。
/// * `Err`: エラー。
pub async fn change_password<'a>(
    repos: Arc<dyn DatabaseService>,
    id: AccountId,
    old_password: &'a str,
    new_password: &'a str,
) -> Result<(), Error> {
    // 古いパスワードを検証
    let old_password = RawPassword::new(old_password);
    if old_password.is_err() {
        return Err(usecase_error(
            ErrorKind::InvalidOldPassword,
            "古いパスワードが不正です。".into(),
        ));
    }
    let old_password = old_password.unwrap();
    // 新しいパスワードを検証
    let new_password = RawPassword::new(new_password);
    if new_password.is_err() {
        return Err(usecase_error(
            ErrorKind::InvalidNewPassword,
            "新しいパスワードが不正です。".into(),
        ));
    }
    let new_password = new_password.unwrap();
    // トランザクションを開始
    let txn = begin_transaction(&repos.connection()).await?;
    {
        // パスワードを変更するアカウントを取得
        let account = find_account(&*repos, &txn, id.clone()).await?;
        // パスワードが一致することを確認
        let result = verify_password(&old_password.value(), &account.password().value());
        if let Err(err) = result {
            return Err(internal_error(err.into()));
        }
        if !result.unwrap() {
            return Err(Error {
                code: ErrorKind::WrongPassword,
                message: "古いパスワードが間違っています。".into(),
            });
        }
        // パスワードをハッシュ化
        let hashed_password = HashedPassword::new(new_password);
        // パスワードを変更
        let result = repos
            .account(&txn)
            .change_password(id, hashed_password)
            .await;
        if let Err(err) = result {
            return Err(internal_error(err.into()));
        }
    }
    // トランザクションをコミット
    match txn.commit().await {
        Ok(_) => Ok(()),
        Err(err) => Err(internal_error(err.into())),
    }
}
