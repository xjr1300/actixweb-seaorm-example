use async_trait::async_trait;

use crate::models::accounts::AccountId;
use crate::models::auth::{JwtTokens, JwtTokensId};

/// 有効期限付きアクセス・リフレッシュトークンリポジトリ
#[async_trait]
pub trait JwtTokensRepository {
    /// トークンIDを指定して、有効期限付きアクセス・リフレッシュトークンを検索する。
    ///
    /// # Arguments
    ///
    /// * `id` - トークンID。
    ///
    /// # Returns
    ///
    /// `Result`。返却される`Result`の内容は以下の通り。
    ///
    /// * `Ok`: 見つかった場合は有効期限付きアクセス・リフレッシュトークン。見つからなかった場合は`None`。
    /// * `Err`: エラー。
    async fn find_by_id(&self, id: JwtTokensId) -> anyhow::Result<Option<JwtTokens>>;

    /// アクセストークンを指定して、有効期限付きアクセス・リフレッシュトークンを検索する。
    ///
    /// # Arguments
    ///
    /// * `token` - アクセストークン。
    ///
    /// # Returns
    ///
    /// `Result`。返却される`Result`の内容は以下の通り。
    ///
    /// * `Ok`: 見つかった場合は有効期限付きアクセス・リフレッシュトークン。見つからなかった場合は`None`。
    /// * `Err`: エラー。
    async fn find_by_access_token(&self, token: &str) -> anyhow::Result<Option<JwtTokens>>;

    /// リフレッシュトークンを指定して、有効期限付きアクセス・リフレッシュトークンを検索する。
    ///
    /// # Arguments
    ///
    /// * `token` - リフレッシュトークン。
    ///
    /// # Returns
    ///
    /// `Result`。返却される`Result`の内容は以下の通り。
    ///
    /// * `Ok`: 見つかった場合は有効期限付きアクセス・リフレッシュトークン。見つからなかった場合は`None`。
    /// * `Err`: エラー。
    async fn find_by_refresh_token(&self, token: &str) -> anyhow::Result<Option<JwtTokens>>;

    /// 有効期限付きアクセス・リフレッシュトークンを登録する。
    ///
    /// # Arguments
    ///
    /// * `tokens` - 有効期限付きアクセス・リフレッシュトークン。
    ///
    /// # Returns
    ///
    /// `Result`。返却される`Result`の内容は以下の通り。
    ///
    /// * `Ok`: 登録した有効期限付きアクセス・リフレッシュトークン。
    /// * `Err`: エラー。
    async fn insert(&self, tokens: &JwtTokens) -> anyhow::Result<JwtTokens>;

    /// 有効期限付きアクセス・リフレッシュトークンを削除する。
    ///
    /// アカウントIDが一致するアクセス・リフレッシュトークンが登録されていない場合は`OK(())`を返却する。
    ///
    /// # Arguments
    ///
    /// * `id` - 削除するアカウントのアカウントID。
    ///
    /// # Returns
    ///
    /// `Result`。返却される`Result`の内容は以下の通り。
    ///
    /// * `Ok`: `()`。
    /// * `Err`: エラー。
    async fn delete(&self, id: AccountId) -> anyhow::Result<()>;
}
