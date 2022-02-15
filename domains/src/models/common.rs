use std::marker::PhantomData;

use anyhow::anyhow;
use chrono::{DateTime, FixedOffset, Utc};
use derive_new::new;
use lazy_static::lazy_static;
use regex::Regex;
use serde::Serialize;
use ulid::Ulid;
use validator::Validate;

lazy_static! {
    /// 電話番号の正規表現。
    static ref PHONE_NUMBER_REGEX: Regex = Regex::new(r"^0\d{1,4}-\d{1,4}-\d{4}$").unwrap();
    /// 郵便番号の正規表現
    static ref POSTAL_CODE_REGEX: Regex = Regex::new(r"^\d{3}-\d{4}$").unwrap();
}

/// エンティティID構造体
///
/// # Description
///
/// エンティティIDは値オブジェクトで、`value`フィールドにエンティティを識別するIDを記録する。
/// `value`フィールドには`ULID`値が記録される。
///
/// # Type Parameters
///
/// * `T`: エンティティの型。
#[derive(new, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EntityId<T> {
    /// エンティティID。
    pub value: Ulid,
    /// エンティティIDがエンティティに属するか識別するデータ。
    _marker: PhantomData<T>,
}

impl<T> EntityId<T> {
    /// エンティティIDを構築する。
    ///
    /// # Returns
    ///
    /// * エンティティID。
    pub fn gen() -> EntityId<T> {
        Self::new(Ulid::new())
    }
}

impl<T> TryFrom<String> for EntityId<T> {
    type Error = anyhow::Error;

    /// 文字列からエンティティIDを構築して返却する。
    ///
    /// # Arguments
    ///
    /// * `value` - エンティティIDを構築する文字列。
    fn try_from(value: String) -> anyhow::Result<Self, Self::Error> {
        Ulid::from_string(&value)
            .map(|id| Self::new(id))
            .map_err(|err| anyhow!("{:?}", err))
    }
}

#[cfg(test)]
mod entity_id_tests {
    use super::*;

    /// ULIDからエンティティIDを構築できることを確認する。
    #[test]
    fn entity_id_from_ulid() {
        let value = Ulid::new();
        let id = EntityId::<i32>::new(value);
        assert_eq!(id.value, value);
    }

    /// ULID文字列からエンティティIDを構築できることを確認する。
    #[test]
    fn entity_id_from_string() {
        // cSpell: ignore 01D39ZY06FGSCTVN4T2V9PKHFZ
        let id = EntityId::<i32>::try_from(String::from("01D39ZY06FGSCTVN4T2V9PKHFZ"));
        assert!(id.is_ok());
    }

    /// ULID文字列以外の文字列からエンティティIDを構築できないことを確認する。
    #[test]
    fn entity_id_from_invalid_string() {
        let id = EntityId::<i32>::try_from(String::from("invalid-ulid-string"));
        assert!(id.is_err());
    }
}

/// Eメールアドレス構造体
#[derive(Debug, Clone, Validate)]
pub struct EmailAddress {
    /// Eメールアドレス。
    #[validate(email)]
    value: String,
}

impl EmailAddress {
    /// コンストラクタ。
    ///
    /// # Arguments
    ///
    /// * `value` - Eメールアドレス。
    ///
    /// # Returns
    ///
    /// `Result`。`Result`の内容は以下の通り。
    ///
    /// * `Ok`: Eメールアドレス構造体。
    /// * `Err`: エラーメッセージ。
    pub fn new(value: &str) -> anyhow::Result<Self> {
        let result = Self {
            value: value.to_owned(),
        };
        if result.validate().is_err() {
            return Err(anyhow!(format!("Eメールアドレス({})が不正です。", value)));
        }

        Ok(result)
    }

    /// Eメールアドレスを文字列で返却する。
    ///
    /// # Returns
    ///
    /// * Eメールアドレス。
    pub fn value(&self) -> String {
        self.value.clone()
    }
}

#[cfg(test)]
mod email_address_tests {
    use super::*;

    /// Eメールアドレスを構築できることを確認する。
    #[test]
    fn test_email_address_new() {
        let value = "email@example.com";
        let result = EmailAddress::new(value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().value(), value);
    }

    /// Eメールアドレスを構築できないことを確認する。
    #[test]
    fn test_email_address_new_invalid() {
        assert!(EmailAddress::new("@example.com").is_err());
    }
}

/// 電話番号構造体
#[derive(Debug, Clone, Validate)]
pub struct PhoneNumber {
    /// 電話番号。
    #[validate(regex = "PHONE_NUMBER_REGEX")]
    value: String,
}

impl PhoneNumber {
    /// コンストラクタ。
    ///
    /// # Arguments
    ///
    /// * `value` - 電話番号。
    ///
    /// # Returns
    ///
    /// `Result`。`Result`の内容は以下の通り。
    ///
    /// * `Ok`: Eメールアドレス構造体。
    /// * `Err`: エラーメッセージ。
    pub fn new(value: &str) -> anyhow::Result<Self> {
        let result = Self {
            value: value.to_owned(),
        };
        if result.validate().is_err() {
            return Err(anyhow!(format!("電話番号({})が不正です。", value)));
        }

        Ok(result)
    }

    /// 電話番号を返却する。
    ///
    /// # Returns
    ///
    /// * 電話番号。
    pub fn value(&self) -> String {
        self.value.clone()
    }
}

#[cfg(test)]
mod phone_number_tests {
    use super::*;

    /// 電話番号を構築できることを確認する。
    #[test]
    fn test_phone_number_new() {
        let valid_number = "012-345-6789";
        let result = PhoneNumber::new(valid_number);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().value(), valid_number);
    }

    /// 電話番号を構築できないことを確認する。
    #[test]
    fn test_phone_number_new_invalid() {
        assert!(PhoneNumber::new("999-9999-9999").is_err());
    }
}

/// 郵便番号構造体
#[derive(Debug, Clone, Validate)]
pub struct PostalCode {
    /// 郵便番号。
    #[validate(regex = "POSTAL_CODE_REGEX")]
    value: String,
}

impl PostalCode {
    /// コンストラクタ。
    ///
    /// # Arguments
    ///
    /// * `value` - 郵便番号。
    ///
    /// # Returns
    ///
    /// `Result`。`Result`の内容は以下の通り。
    ///
    /// * `Ok`: Eメールアドレス構造体。
    /// * `Err`: エラーメッセージ。
    pub fn new(value: &str) -> anyhow::Result<Self> {
        let result = PostalCode {
            value: value.to_owned(),
        };
        if result.validate().is_err() {
            return Err(anyhow::anyhow!(format!("郵便番号({})が不正です。", value)));
        }

        Ok(result)
    }

    /// 郵便番号を返却する。
    ///
    /// # Returns
    ///
    /// * 郵便番号。
    pub fn value(&self) -> String {
        self.value.clone()
    }
}

#[cfg(test)]
mod postal_code_tests {
    use super::*;

    /// 郵便番号を構築できることを確認する。
    #[test]
    fn test_postal_code_new() {
        let valid_code = "500-8570";
        let result = PostalCode::new(valid_code);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().value(), valid_code);
    }

    /// 郵便番号を構築できないことを確認する。
    #[test]
    fn test_postal_code_new_invalid() {
        assert!(PostalCode::new("00-0000").is_err());
    }
}

/// 都道府県構造体
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Prefecture {
    /// 都道府県コード。
    code: u8,
    /// 都道府県名。
    name: String,
}

impl Prefecture {
    /// コンストラクタ。
    ///
    /// # Returns
    ///
    /// * 都道府県。
    pub fn new(code: u8, name: &str) -> Self {
        Self {
            code,
            name: name.to_owned(),
        }
    }

    /// 都道府県コードを返却する。
    ///
    /// # Returns
    ///
    /// * 都道府県コード。
    pub fn code(&self) -> u8 {
        self.code
    }

    /// 都道府県名を返却する。
    ///
    /// # Returns
    ///
    /// * 都道府県コード。
    pub fn name(&self) -> String {
        self.name.clone()
    }
}

#[cfg(test)]
mod prefecture_tests {
    use super::*;

    /// 都道府県を構築できることを確認する。
    #[test]
    fn test_prefecture_new() {
        let code = 12;
        let name = "東京都";
        let prefecture = Prefecture::new(code, name);
        assert_eq!(prefecture.code(), code);
        assert_eq!(prefecture.name(), name);
    }
}

/// 市区町村以下住所構造体。
///
/// 市町村以下の住所は2文字以上100文字以下の文字列を記録する。
#[derive(Debug, Clone, Validate)]
pub struct AddressDetails {
    #[validate(length(min = 2, max = 100))]
    value: String,
}

impl AddressDetails {
    /// コンストラクタ。
    ///
    /// # Arguments
    ///
    /// * `value` - 市区町村以下の住所。
    ///
    /// # Returns
    ///
    /// `Result`。`Result`の内容は以下の通り。
    ///
    /// * `Ok`: 市区町村以下住所。
    /// * `Err`: エラーメッセージ。
    pub fn new(value: &str) -> anyhow::Result<Self> {
        let result = Self {
            value: value.to_owned(),
        };
        if result.validate().is_err() {
            return Err(anyhow!(format!(
                "市区町村以下住所({})は{}文字以上{}文字以下です。",
                value, 2, 100
            )));
        }

        Ok(result)
    }

    /// 市区町村以下住所を返却する。
    ///
    /// # Returns
    ///
    /// * 市区町村以下住所。
    #[allow(dead_code)]
    pub fn value(&self) -> String {
        self.value.clone()
    }
}

#[cfg(test)]
mod address_details_tests {
    use super::*;

    /// 市区町村以下住所を構築できることを確認する。
    #[test]
    fn test_address_details_new() {
        let vec_details = vec!["新宿区西新宿2-8-1", "新宿"];
        for details in vec_details {
            let result = AddressDetails::new(details);
            assert!(result.is_ok());
            assert_eq!(result.unwrap().value(), details);
        }
    }

    /// 市区町村以下住所を構築できないことを確認する。
    #[test]
    fn test_address_details_new_invalid() {
        assert!(PostalCode::new("0").is_err());
        assert!(PostalCode::new(&"0".repeat(101)).is_err());
    }
}

/// 住所構造体
#[derive(Debug, Clone)]
pub struct Address {
    /// 都道府県。
    prefecture: Prefecture,
    /// 市区町村以下の住所。
    details: AddressDetails,
}

impl Address {
    /// コンストラクタ。
    ///
    /// # Arguments
    ///
    /// * `prefecture` - 都道府県。
    /// * `details` - 市区町村以下の住所。
    ///
    /// # Returns
    ///
    /// * 住所。
    pub fn new(prefecture: Prefecture, details: AddressDetails) -> Self {
        Self {
            prefecture,
            details,
        }
    }

    /// 都道府県を返却する。
    ///
    /// # Returns
    ///
    /// * 都道府県。
    pub fn prefecture(&self) -> Prefecture {
        self.prefecture.clone()
    }

    /// 市区町村以下の住所を返却する。
    ///
    /// # Returns
    ///
    /// * 市区町村以下の住所。
    pub fn details(&self) -> AddressDetails {
        self.details.clone()
    }
}

#[cfg(test)]
mod address_tests {
    use super::*;

    /// 住所を構築できることを確認する。
    #[test]
    fn test_address_new() {
        let pref_code = 13;
        let pref_name = "東京都";
        let prefecture = Prefecture::new(pref_code, pref_name);
        let address_details = AddressDetails::new("新宿区西新宿2-8-1").unwrap();
        let address = Address::new(prefecture, address_details.clone());
        assert_eq!(address.prefecture().code(), pref_code);
        assert_eq!(address.prefecture().name(), pref_name);
        assert_eq!(address.details().value(), address_details.value());
    }
}

/// 日本標準時の現在日時を返却する。
///
/// # Returns
///
/// * 日本標準時の現在日時。
///
/// # Example
///
/// ```rust
/// use chrono::{DateTime, Utc, FixedOffset};
/// use domains::models::common::local_now;
///
/// let utc = Utc::now();
/// let local = local_now(Some(utc));
/// assert_eq!(utc, local);
/// ```
pub fn local_now(utc: Option<DateTime<Utc>>) -> DateTime<FixedOffset> {
    let offset = FixedOffset::east(9 * 60 * 60);
    let utc = utc.unwrap_or_else(Utc::now);

    utc.with_timezone(&offset)
}
