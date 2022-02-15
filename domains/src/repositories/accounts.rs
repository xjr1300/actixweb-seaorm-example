use async_trait::async_trait;

use crate::models::accounts::{Account, AccountId, HashedPassword};
use crate::models::common::EmailAddress;

/// アカウントリポジトリ
#[async_trait]
pub trait AccountRepository {
    /// アカウントIDを指定して、アカウントを検索する。
    ///
    /// # Arguments
    ///
    /// * `id` - アカウントID。
    ///
    /// # Returns
    ///
    /// `Result`。返却される`Result`の内容は以下の通り。
    ///
    /// * `Ok`: アカウントが見つかった場合はアカウント。アカウントが見つからなかった場合は`None`。
    /// * `Err`: エラーメッセージ。
    async fn find_by_id(&self, id: AccountId) -> anyhow::Result<Option<Account>>;

    /// Eメールを指定して、アカウントを検索する。
    ///
    /// # Arguments
    ///
    /// * `email` - Eメールアドレス。
    ///
    /// # Returns
    ///
    /// `Result`。返却される`Result`の内容は以下の通り。
    ///
    /// * `Ok`: アカウントが見つかった場合はアカウント。アカウントが見つからなかった場合は`None`。
    /// * `Err`: エラーメッセージ。
    async fn find_by_email(&self, email: EmailAddress) -> anyhow::Result<Option<Account>>;

    /// アカウントのリストを返却する。
    ///
    /// # Returns
    ///
    /// `Result`。返却される`Result`の内容は以下の通り。
    ///
    /// * `Ok`: アカウントを格納したベクタ。
    /// * `Err`: エラーメッセージ。
    async fn list(&self) -> anyhow::Result<Vec<Account>>;

    /// アカウントを登録する。
    ///
    /// # Arguments
    ///
    /// * `account` - アカウント。
    ///
    /// # Result
    ///
    /// `Result`。返却される`Result`の内容は以下の通り。
    ///
    /// * `Ok`: 登録したアカウント。
    /// * `Err`: エラー。
    async fn insert(&self, account: &Account) -> anyhow::Result<Account>;

    /// アカウントを更新する。
    ///
    /// # Arguments
    ///
    /// * `account` - アカウント。
    ///
    /// # Result
    ///
    /// `Result`。返却される`Result`の内容は以下の通り。
    ///
    /// * `Ok`: 更新後のアカウント。
    /// * `Err`: エラー。
    async fn update(&self, account: &Account) -> anyhow::Result<Account>;

    /// アカウントを削除する。
    ///
    /// アカウントIDが一致するアカウントが登録されていない場合は`OK(())`を返却する。
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

    /// パスワードを変更する。
    ///
    /// # Arguments
    ///
    /// * `id` - パスワードを変更するアカウントのアカウントID。
    /// * `password` - 新たに設定するハッシュ化したパスワード。
    ///
    /// # Returns
    ///
    /// `Result`。返却される`Result`の内容は下記の通り。
    ///
    /// * `Ok`: パスワードの変更に成功した場合は`true`。アカウントが見つからない場合は`false`。
    /// * `Err`: エラー。
    async fn change_password(
        &self,
        id: AccountId,
        new_password: HashedPassword,
    ) -> anyhow::Result<bool>;
}
