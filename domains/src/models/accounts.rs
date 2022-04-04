use anyhow::anyhow;
use chrono::{DateTime, FixedOffset};
use validator::Validate;

use super::common::{local_now, Address, EmailAddress, EntityId, PhoneNumber, PostalCode};

/// アカウントID型
pub type AccountId = EntityId<Account>;

/// アカウント名の文字列の長さ
const ACCOUNT_NAME_MIN_LENGTH: usize = 2;
const ACCOUNT_NAME_MAX_LENGTH: usize = 20;

/// パスワードの最小文字数
const RAW_PASSWORD_MIN_LENGTH: usize = 8;
// パスワードに使用できる文字
const RAW_PASSWORD_SIGNS: &str = r##" !"#$%&'()*+,-./:;<=>?@[\]^_`{|}~"##;

/// アカウント名構造体
///
/// アカウント名は2文字以上かつ20文字以下までの文字列を受け付ける。
#[derive(Debug, Clone, Validate)]
pub struct AccountName {
    #[validate(length(min = "ACCOUNT_NAME_MIN_LENGTH", max = "ACCOUNT_NAME_MAX_LENGTH"))]
    value: String,
}

impl AccountName {
    /// コンストラクタ。
    ///
    /// # Arguments
    ///
    /// * `value` - アカウント名。
    ///
    /// # Returns
    ///
    /// `Result`。`Result`の内容は以下の通り。
    ///
    /// * `Ok`: アカウント名。
    /// * `Err`: エラーメッセージ。
    pub fn new(value: &str) -> anyhow::Result<Self> {
        let result = Self {
            value: value.to_owned(),
        };
        if result.validate().is_err() {
            return Err(anyhow!(format!(
                "アカウント名({})は{}以上{}以下の文字列を指定してください。",
                value, ACCOUNT_NAME_MIN_LENGTH, ACCOUNT_NAME_MAX_LENGTH
            )));
        }

        Ok(result)
    }

    /// アカウント名を文字列で返却する。
    ///
    /// # Returns
    ///
    /// アカウント名を示す文字列。
    pub fn value(&self) -> String {
        self.value.clone()
    }
}

#[cfg(test)]
mod account_name_tests {
    use super::*;

    /// アカウント名を構築できることを確認する。
    #[test]
    fn test_account_name_new() {
        let valid_names = vec![
            "0".repeat(ACCOUNT_NAME_MIN_LENGTH),
            "0".repeat(ACCOUNT_NAME_MAX_LENGTH),
        ];
        for name in valid_names {
            let result = AccountName::new(&name);
            assert!(result.is_ok());
            assert_eq!(result.unwrap().value(), name);
        }
    }

    /// アカウント名を構築できないことを確認する。
    #[test]
    fn test_account_name_new_invalid() {
        #[allow(clippy::all)]
        let invalid_names = vec![
            "0".repeat(ACCOUNT_NAME_MIN_LENGTH - 1),
            "0".repeat(ACCOUNT_NAME_MAX_LENGTH + 1),
        ];
        for name in invalid_names {
            let result = AccountName::new(&name);
            assert!(result.is_err());
        }
    }
}

/// パスワード構造体
///
/// パスワードは、アルファベットの大文字と小文字、数字及び記号で構成された、8文字以上の文字列
/// でなければならない。
#[derive(Debug, Clone, Validate)]
pub struct RawPassword {
    #[validate(length(min = "RAW_PASSWORD_MIN_LENGTH"))]
    value: String,
}

impl RawPassword {
    /// コンストラクタ。
    ///
    /// # Arguments
    ///
    /// * `value` - パスワード。
    ///
    /// # Returns
    ///
    /// `Result`。返却される`Result`の内容は以下の通り。
    ///
    /// * `Ok`: パスワード。
    /// * `Err`: エラーメッセージ。
    pub fn new(value: &str) -> anyhow::Result<Self> {
        let result = Self {
            value: value.to_owned(),
        };
        if result.validate().is_err() {
            return Err(anyhow!(format!(
                "パスワードは{}文字以上の文字列で指定してください。",
                RAW_PASSWORD_MIN_LENGTH
            )));
        }
        if !value.chars().any(|ch| ch.is_ascii_alphabetic()) {
            return Err(anyhow!("パスワードにアルファベットが含まれていません。"));
        }
        if !value.chars().any(|ch| ch.is_ascii_lowercase()) {
            return Err(anyhow!(
                "パスワードに小文字のアルファベットが含まれていません。"
            ));
        }
        if !value.chars().any(|ch| ch.is_ascii_uppercase()) {
            return Err(anyhow!(
                "パスワードに大文字のアルファベットが含まれていません。"
            ));
        }
        if !value.chars().any(|ch| ch.is_ascii_digit()) {
            return Err(anyhow!("パスワードに数字が含まれていません。"));
        }
        if !value.chars().any(|ch| RAW_PASSWORD_SIGNS.contains(ch)) {
            return Err(anyhow!("パスワードに記号が含まれていません。"));
        }

        Ok(result)
    }

    /// パスワードを返却する。
    ///
    /// # Returns
    ///
    /// * パスワード。
    pub fn value(&self) -> String {
        self.value.clone()
    }
}

#[cfg(test)]
mod raw_password_tests {
    use super::*;

    /// パスワードを構築できることを確認する。
    #[test]
    fn test_raw_password_new() {
        let valid_password = "01abCD#$";
        let result = RawPassword::new(valid_password);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().value(), valid_password);
    }

    /// パスワードを構築できないことを確認する。
    #[test]
    fn test_raw_password_new_invalid() {
        // 7文字
        assert!(RawPassword::new("01abCD#").is_err());
        // アルファベットを含んでいない
        assert!(RawPassword::new("012345#$").is_err());
        // 大文字のファルファベットを含んでいない
        assert!(RawPassword::new("01abcd#$").is_err());
        // 小文字のファルファベットを含んでいない
        assert!(RawPassword::new("01ABCD#$").is_err());
        // 数字を含んでいない
        assert!(RawPassword::new("012346#$").is_err());
        // 記号を含んでいない
        assert!(RawPassword::new("01abCDef").is_err());
    }
}

/// ハッシュ化パスワード構造体
#[derive(Debug, Clone)]
pub struct HashedPassword {
    /// ハッシュ化パスワード。
    value: String,
}

impl HashedPassword {
    /// コンストラクタ。
    ///
    /// # Arguments
    ///
    /// * `raw` - パスワード。
    ///
    /// # Returns
    ///
    /// * ハッシュ化したパスワード。
    pub fn new(raw: RawPassword) -> Self {
        use crate::services::hashers::{hash_password, SaultProviderImpl};

        let sault_provider = SaultProviderImpl {};

        Self {
            value: hash_password(&sault_provider, &raw.value).unwrap(),
        }
    }

    /// コンストラクタ。
    ///
    /// この関連関数はリポジトリから呼び出すこと。
    /// リポジトリ以外からは呼び出してはならない。
    ///
    /// # Arguments
    ///
    /// * `value` - ハッシュ化されたパスワード。
    ///
    /// # Returns
    ///
    /// ハッシュ化パスワード。
    pub fn from_repository(value: &str) -> Self {
        Self {
            value: value.to_owned(),
        }
    }

    /// ハッシュ化したパスワードを返却する。
    ///
    /// # Returns
    ///
    /// * ハッシュ化したパスワード。
    pub fn value(&self) -> String {
        self.value.clone()
    }
}

#[cfg(test)]
mod hashed_password_tests {
    use super::*;

    /// ハッシュ化したパスワードをチェックなしで構築できるか確認する。
    #[test]
    fn test_hashed_password_new_unchecked() {
        let hashed = "this-is-hashed-password";
        let value = HashedPassword::from_repository(hashed);
        assert_eq!(value.value(), hashed);
    }
}

/// 文字列から電話番号に変換する。
///
/// # Arguments
///
/// * `value` - 電話番号。
///
/// # Returns
///
/// `Result`。`Result`の内容は以下の通り。
///
/// * `Option<PhoneNumber>` - 電話番号または引数が`None`の場合は`None`。
/// * `Err` - エラーメッセージ。
pub fn optional_phone_number(value: Option<&str>) -> anyhow::Result<Option<PhoneNumber>> {
    match value {
        Some(value) => {
            let value = PhoneNumber::new(value)?;
            Ok(Some(value))
        }
        None => Ok(None),
    }
}

/// 電話番号を文字列に変換する。
///
/// # Arguments
///
/// * `value` - 電話番号。
///
/// # Returns
///
/// `Option`。`Option`の内容は以下の通り。
///
/// * `Some`: 電話番号文字列。
/// * `None`: 電話番号が設定されていない場合は`None`。
pub fn optional_phone_number_string(value: Option<PhoneNumber>) -> Option<String> {
    value.map(|value| value.value())
}

#[cfg(test)]
mod optional_phone_number_tests {
    use super::*;

    /// 有効な文字列から電話番号を構築できるか確認する。
    #[test]
    fn test_optional_phone_number() {
        let value = "012-345-6789";
        let result = optional_phone_number(Some(value));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().unwrap().value(), value);
    }

    /// Noneの場合にNoneを返却するか確認する。
    #[test]
    fn test_optional_phone_number_none() {
        let result = optional_phone_number(None);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    /// 無効な文字列から電話番号を構築できないことを確認する。
    #[test]
    fn test_optional_phone_number_invalid() {
        let result = optional_phone_number(Some("invalid-number"));
        assert!(result.is_err());
    }
}

/// 固定携帯電話番号構造体
///
/// 固定電話番号または携帯電話番号のうち、とちらかの電話番号を記録する必要がある。
#[derive(Debug, Clone)]
pub struct FixedMobileNumbers {
    /// 固定電話番号。
    fixed: Option<PhoneNumber>,
    /// 携帯電話番号。
    mobile: Option<PhoneNumber>,
}

impl FixedMobileNumbers {
    /// コンストラクタ。
    ///
    /// # Arguments
    ///
    /// * `fixed` - 固定電話番号。
    /// * `mobile` - 携帯電話番号。
    ///
    /// # Returns
    ///
    /// `Result`。`Result`の内容は以下の通り。
    ///
    /// * `Ok`: 固定携帯電話番号。
    /// * `Err`: エラーメッセージ。
    pub fn new(
        fixed: Option<PhoneNumber>,
        mobile: Option<PhoneNumber>,
    ) -> anyhow::Result<FixedMobileNumbers> {
        if fixed.is_none() && mobile.is_none() {
            return Err(anyhow!(
                "少なくとも固定電話番号か携帯電話番号に、電話番号を設定する必要があります。"
            ));
        }

        Ok(Self { fixed, mobile })
    }

    /// 固定電話番号を返却する。
    ///
    /// # Returns
    ///
    /// * 固定電話番号。
    pub fn fixed(&self) -> Option<PhoneNumber> {
        self.fixed.clone()
    }

    /// 携帯電話番号を返却する。
    ///
    /// # Returns
    ///
    /// * 携帯電話番号。
    pub fn mobile(&self) -> Option<PhoneNumber> {
        self.mobile.clone()
    }
}

#[cfg(test)]
mod fixed_mobile_phone_numbers_tests {
    use super::*;

    /// 固定携帯電話番号を構築できることを確認する。
    #[test]
    fn test_fixed_mobile_phone_numbers_new() {
        let fixed = Some(PhoneNumber::new("012-345-6789").unwrap());
        let mobile = Some(PhoneNumber::new("090-1234-5678").unwrap());
        let result = FixedMobileNumbers::new(fixed.clone(), mobile.clone());
        assert!(result.is_ok());
        assert_eq!(
            result.as_ref().unwrap().fixed().unwrap().value(),
            fixed.clone().unwrap().value()
        );
        assert_eq!(
            result.as_ref().unwrap().mobile().unwrap().value(),
            mobile.clone().unwrap().value()
        );
        assert!(FixedMobileNumbers::new(fixed, None).is_ok());
        assert!(FixedMobileNumbers::new(None, mobile).is_ok());
    }

    /// 固定携帯電話番号を構築できないことを確認する。
    #[test]
    fn test_fixed_mobile_phone_numbers_new_invalid() {
        assert!(FixedMobileNumbers::new(None, None).is_err());
    }
}

/// アカウント
///
/// アカウントが有効であるかは、`active`フィールドで判断する。
#[derive(Debug, Clone)]
pub struct Account {
    /// アカウントID。
    id: AccountId,
    /// Eメールアドレス。
    email: EmailAddress,
    /// アカウント名。
    name: AccountName,
    /// ハッシュ化済パスワード。
    password: HashedPassword,
    /// アクティブフラグ。
    is_active: bool,
    /// 固定携帯電話番号。
    phone_numbers: FixedMobileNumbers,
    /// 郵便番号。
    postal_code: PostalCode,
    /// 住所。
    address: Address,
    /// 最終ログイン日時。
    logged_in_at: Option<DateTime<FixedOffset>>,
    /// 作成日時。
    created_at: DateTime<FixedOffset>,
    /// 更新日時。
    updated_at: DateTime<FixedOffset>,
}

impl Account {
    /// コンストラクタ。
    ///
    /// # Arguments
    ///
    /// * `email` - Eメールアドレス。
    /// * `name` - アカウント名。
    /// * `password` - パスワード。
    /// * `is_active` - アクティブフラグ。
    /// * `phone_numbers` - 固定携帯電話番号。
    /// * `postal_code` - 郵便番号。
    /// * `address` - 住所。
    ///
    /// # Returns
    ///
    /// * アカウント。
    pub fn new(
        email: EmailAddress,
        name: AccountName,
        password: RawPassword,
        is_active: bool,
        phone_numbers: FixedMobileNumbers,
        postal_code: PostalCode,
        address: Address,
    ) -> Self {
        let dt = local_now(None);

        Self {
            id: AccountId::gen(),
            email,
            name,
            password: HashedPassword::new(password),
            is_active,
            phone_numbers,
            postal_code,
            address,
            logged_in_at: None,
            created_at: dt,
            updated_at: dt,
        }
    }

    /// コンストラクタ。
    ///
    /// この関連関数はリポジトリから呼び出すこと。
    /// リポジトリ以外からは呼び出してはならない。
    ///
    /// # Arguments
    ///
    /// * `id` - アカウントID。
    /// * `email` - Eメールアドレス。
    /// * `name` - アカウント名。
    /// * `password` - ハッシュ化されたパスワード。
    /// * `is_active` - アクティブフラグ。
    /// * `phone_numbers` - 固定携帯電話番号。
    /// * `postal_code` - 郵便番号。
    /// * `address` - 住所。
    /// * `logged_in_at` - 最終ログイン日時。
    /// * `created_at` - 登録日時。
    /// * `updated_at` - 更新日時。
    ///
    /// # Returns
    ///
    /// * アカウント。
    #[allow(clippy::too_many_arguments)]
    pub fn new_unchecked(
        id: AccountId,
        email: EmailAddress,
        name: AccountName,
        password: HashedPassword,
        is_active: bool,
        phone_numbers: FixedMobileNumbers,
        postal_code: PostalCode,
        address: Address,
        logged_in_at: Option<DateTime<FixedOffset>>,
        created_at: DateTime<FixedOffset>,
        updated_at: DateTime<FixedOffset>,
    ) -> Self {
        Self {
            id,
            email,
            name,
            password,
            is_active,
            phone_numbers,
            postal_code,
            address,
            logged_in_at,
            created_at,
            updated_at,
        }
    }

    /// アカウントIDを返却する。
    ///
    /// # Returns
    ///
    /// * アカウントID。
    pub fn id(&self) -> AccountId {
        self.id.clone()
    }

    /// Eメールアドレスを返却する。
    ///
    /// # Returns
    ///
    /// * Eメールアドレスを返却する。
    pub fn email(&self) -> EmailAddress {
        self.email.clone()
    }

    /// アカウント名を返却する。
    ///
    /// # Returns
    ///
    /// * アカウント名。
    pub fn name(&self) -> AccountName {
        self.name.clone()
    }

    /// アカウント名を設定する。
    ///
    /// # Argument
    ///
    /// * `value`: アカウント名。
    pub fn set_name(&mut self, value: AccountName) {
        self.name = value;
    }

    /// ハッシュ化済パスワードを返却する。
    ///
    /// # Returns
    ///
    /// * ハッシュ化済パスワード。
    pub fn password(&self) -> HashedPassword {
        self.password.clone()
    }

    /// アカウントが有効かどうかを返却する。
    ///
    /// # Returns
    ///
    /// `true`の場合はアカウントが有効。`false`の場合はアカウントが無効。
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// アカウントが有効化どうかを設定する。
    ///
    /// # Arguments
    ///
    /// * `value`: `true`の場合はアカウントが有効。`false`の場合はアカウントが無効。
    pub fn set_is_active(&mut self, value: bool) {
        self.is_active = value;
    }

    /// 固定携帯電話番号を返却する。
    ///
    /// # Returns
    ///
    /// * 固定携帯電話番号。
    pub fn phone_numbers(&self) -> FixedMobileNumbers {
        self.phone_numbers.clone()
    }

    /// 固定携帯電話番号を設定する。
    ///
    /// # Arguments
    ///
    /// * `value` - 固定携帯電話番号。
    pub fn set_phone_numbers(&mut self, value: FixedMobileNumbers) {
        self.phone_numbers = value;
    }

    /// 郵便番号を返却する。
    ///
    /// # Returns
    ///
    /// * 郵便番号。
    pub fn postal_code(&self) -> PostalCode {
        self.postal_code.clone()
    }

    /// 郵便番号を設定する。
    ///
    /// # Arguments
    ///
    /// * `value` - 郵便番号。
    pub fn set_postal_code(&mut self, value: PostalCode) {
        self.postal_code = value;
    }

    /// 住所を返却する。
    ///
    /// # Returns
    ///
    /// * 住所。
    pub fn address(&self) -> Address {
        self.address.clone()
    }

    /// 住所を設定する。
    ///
    /// # Arguments
    ///
    /// * `value` - 住所。
    pub fn set_address(&mut self, value: Address) {
        self.address = value;
    }

    /// 最終ログイン日時を返却する。
    ///
    /// # Returns
    ///
    /// * 最終ログイン日時。
    /// * ログインしていない場合は`None`。
    pub fn logged_in_at(&self) -> Option<DateTime<FixedOffset>> {
        self.logged_in_at
    }

    /// 最終ログイン日時を設定する。
    ///
    /// # Arguments
    ///
    /// * `value` - 最終ログイン日時。ログインしていない場合は`None`。
    pub fn set_logged_in_at(&mut self, value: Option<DateTime<FixedOffset>>) {
        self.logged_in_at = value;
    }

    /// 作成日時を返却する。
    ///
    /// # Returns
    ///
    /// * 作成日時。
    pub fn created_at(&self) -> DateTime<FixedOffset> {
        self.created_at
    }

    /// 更新日時を返却する。
    ///
    /// # Returns
    ///
    /// * 更新日時。
    pub fn updated_at(&self) -> DateTime<FixedOffset> {
        self.updated_at
    }

    /// 更新日時を設定する。
    ///
    /// # Arguments
    ///
    /// * `value` - 更新日時。
    pub fn set_updated_at(&mut self, value: DateTime<FixedOffset>) {
        self.updated_at = value;
    }
}

impl PartialEq for Account {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl PartialOrd for Account {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

#[cfg(test)]
mod account_tests {
    use super::super::common::{AddressDetails, Prefecture};
    use super::*;
    use ulid::Ulid;

    /// アカウントを構築できることを確認する。
    #[test]
    fn test_account_new() {
        let email = EmailAddress::new("foo@example.com").unwrap();
        let name = AccountName::new("foo").unwrap();
        let password = RawPassword::new("01abCD#$").unwrap();
        let is_active = true;
        let fixed_number = PhoneNumber::new("012-345-6890").unwrap();
        let mobile_number = PhoneNumber::new("090-1234-5678").unwrap();
        let phone_numbers =
            FixedMobileNumbers::new(Some(fixed_number.clone()), Some(mobile_number.clone()))
                .unwrap();
        let postal_code = PostalCode::new("012-3456").unwrap();
        let pref_code = 13;
        let pref_name = "東京都";
        let prefecture = Prefecture::new(pref_code, pref_name);
        let address_details = AddressDetails::new("新宿区西新宿2-8-1").unwrap();
        let address = Address::new(prefecture.clone(), address_details.clone());
        // アカウントを構築
        let account = Account::new(
            email.clone(),
            name.clone(),
            password.clone(),
            is_active,
            phone_numbers.clone(),
            postal_code.clone(),
            address.clone(),
        );
        assert_eq!(account.email().value(), email.value());
        assert_eq!(account.name().value(), name.value());
        assert_eq!(account.is_active, is_active);
        assert_eq!(
            account.phone_numbers().fixed().unwrap().value(),
            fixed_number.value()
        );
        assert_eq!(account.postal_code().value(), postal_code.value());
        assert_eq!(account.address().prefecture().code(), pref_code);
        assert_eq!(account.address().prefecture().name(), pref_name);
        assert_eq!(account.address().details().value(), address_details.value());
    }

    /// アカウントを構築できることを確認する。
    #[test]
    fn test_account_new_unchecked() {
        let id = Ulid::new();
        let email = EmailAddress::new("foo@example.com").unwrap();
        let name = AccountName::new("foo").unwrap();
        let password = HashedPassword::from_repository("01abCD#$");
        let is_active = true;
        let fixed_number = PhoneNumber::new("012-345-6890").unwrap();
        let mobile_number = PhoneNumber::new("090-1234-5678").unwrap();
        let phone_numbers =
            FixedMobileNumbers::new(Some(fixed_number.clone()), Some(mobile_number.clone()))
                .unwrap();
        let postal_code = PostalCode::new("012-3456").unwrap();
        let pref_code = 13;
        let pref_name = "東京都";
        let prefecture = Prefecture::new(pref_code, pref_name);
        let address_details = AddressDetails::new("新宿区西新宿2-8-1").unwrap();
        let address = Address::new(prefecture.clone(), address_details.clone());
        let logged_in_at = Some(local_now(None));
        let created_at = local_now(None);
        let updated_at = local_now(None);
        // アカウントを構築
        let account = Account::new_unchecked(
            AccountId::new(id),
            email.clone(),
            name.clone(),
            password.clone(),
            is_active,
            phone_numbers.clone(),
            postal_code.clone(),
            address.clone(),
            logged_in_at,
            created_at,
            updated_at,
        );
        assert_eq!(account.id.value, id);
        assert_eq!(account.email().value(), email.value());
        assert_eq!(account.name().value(), name.value());
        assert_eq!(account.is_active, is_active);
        assert_eq!(
            account.phone_numbers().fixed().unwrap().value(),
            fixed_number.value()
        );
        assert_eq!(account.postal_code().value(), postal_code.value());
        assert_eq!(account.address().prefecture().code(), pref_code);
        assert_eq!(account.address().prefecture().name(), pref_name);
        assert_eq!(account.address().details().value(), address_details.value());
        assert_eq!(account.logged_in_at(), logged_in_at);
        assert_eq!(account.created_at, created_at);
        assert_eq!(account.updated_at, updated_at);
    }
}
