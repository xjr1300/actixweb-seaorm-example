use async_trait::async_trait;

use domains::models::{
    accounts::{Account, AccountId},
    auth::JwtTokens,
};

pub struct AccountTokens {
    pub account: Account,
    pub tokens: Option<JwtTokens>,
}

#[async_trait]
pub trait AccountQueryService {
    /// アカウントとトークンを取得する。
    ///
    /// # Arguments
    ///
    /// * `id` - アカウントID。
    ///
    /// # Returns
    ///
    /// `Result`。返却される`Result`の内容は以下の通り。
    ///
    /// * `Ok`: アカウント道買った場合はアカウントとトークン。アカウントが見つからなかった場合は`None`。
    /// * `Err`: エラー。
    async fn find_active_account_by_id(
        &self,
        id: AccountId,
    ) -> anyhow::Result<Option<AccountTokens>>;
}
