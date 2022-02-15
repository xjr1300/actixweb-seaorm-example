use anyhow::anyhow;
use chrono::{DateTime, FixedOffset};
use validator::Validate;

use super::{accounts::AccountId, common::EntityId};

/// JWTトークン構造体
#[derive(Debug, Clone, Validate)]
pub struct JwtToken {
    /// JWTトークン。
    #[validate(length(min = 1))]
    value: String,
}

impl JwtToken {
    /// コンストラクタ。
    ///
    /// # Arguments
    ///
    /// * `value` - トークン。
    ///
    /// # Returns
    ///
    /// `Result`。返却される｀Result`の内容は以下の通り。
    ///
    /// * `Ok`: JTWトークン。
    /// * `Err`: エラーメッセージ。
    pub fn new(value: &str) -> anyhow::Result<Self> {
        let result = Self {
            value: value.to_string(),
        };
        if result.validate().is_err() {
            return Err(anyhow!(format!("トークン({})が不正です。", value)));
        }

        Ok(result)
    }

    /// JWTトークンを文字列で返却する。
    ///
    /// # Returns
    ///
    /// * JWTトークン。
    pub fn value(&self) -> String {
        self.value.clone()
    }
}

#[cfg(test)]
mod jwt_token_tests {
    use super::*;

    /// JWTトークンを構築できることを確認する。
    #[test]
    fn test_jwt_token_new() {
        let token = "t";
        let result = JwtToken::new(token).unwrap();
        assert_eq!(result.value, token);
    }

    /// JWTトークンを構築できないことを確認する。
    #[test]
    fn test_jwt_token_new_invalid() {
        let invalid_token = "";
        assert!(JwtToken::new(invalid_token).is_err());
    }
}

/// 有効期限付きJWTトークン構造体。
#[derive(Debug, Clone)]
pub struct JwtTokenWithExpiredAt {
    /// JWTトークン。
    pub token: JwtToken,
    /// JWTトークンの有効期限。
    pub expired_at: DateTime<FixedOffset>,
}

pub type JwtTokensId = EntityId<JwtTokens>;

/// 有効期限付きJWTアクセス・リフレッシュトークン構造体
///
/// アクセストークンとリフレッシュトークンを管理する。
#[derive(Debug, Clone)]
pub struct JwtTokens {
    /// トークンID。
    id: JwtTokensId,
    /// アカウントID。
    account_id: AccountId,
    /// アクセストークン。
    access: JwtTokenWithExpiredAt,
    /// リフレッシュトークン。
    refresh: JwtTokenWithExpiredAt,
}

impl JwtTokens {
    /// コンストラクタ。
    ///
    /// # Arguments
    ///
    /// * `id` - アカウントID。
    /// * `access` - 有効期限付きアクセストークン。
    /// * `refresh` - 有効期限付きリフレッシュトークン。
    ///
    /// # Returns
    ///
    /// * アクセスリフレッシュトークン。
    pub fn new(
        id: JwtTokensId,
        account_id: AccountId,
        access: JwtTokenWithExpiredAt,
        refresh: JwtTokenWithExpiredAt,
    ) -> Self {
        Self {
            id,
            account_id,
            access,
            refresh,
        }
    }

    /// トークンIDを返却する`。
    pub fn id(&self) -> JwtTokensId {
        self.id.clone()
    }

    /// アカウントIDを返却する。
    ///
    /// # Returns
    ///
    /// アカウントID。
    pub fn account_id(&self) -> AccountId {
        self.account_id.clone()
    }

    /// 有効期限付きアクセストークンを返却する。
    ///
    /// # Returns
    ///
    /// 有効期限付きアクセストークン。
    pub fn access(&self) -> JwtTokenWithExpiredAt {
        self.access.clone()
    }

    /// 有効期限付きリフレッシュトークンを返却する。
    ///
    /// # Returns
    ///
    /// 有効期限付きリフレッシュトークン。
    pub fn refresh(&self) -> JwtTokenWithExpiredAt {
        self.refresh.clone()
    }
}
