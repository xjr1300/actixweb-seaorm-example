use async_trait::async_trait;

use crate::models::common::Prefecture;

/// 都道府県リポジトリ
#[async_trait]
pub trait PrefectureRepository {
    /// 都道府県コードを指定して、都道府県を検索する。
    ///
    /// # Arguments
    ///
    /// * `code` - 都道府県コード。
    ///
    /// # Returns
    ///
    /// `Result`。返却される`Result`の内容は以下の通り。
    ///
    /// * `Ok`: 都道府県が見つかった場合は都道府県。都道府県が見つからなかった場合は`None`。
    /// * `Err`: エラーメッセージ。
    async fn find_by_code(&self, code: u8) -> anyhow::Result<Option<Prefecture>>;

    /// 都道府県のリストを返却する。
    ///
    /// # Returns
    ///
    /// `Result`。返却される`Result`の内容は以下の通り。
    ///
    /// * `Ok`: 都道府県を格納したベクタ。
    /// * `Err`: エラーメッセージ。
    async fn list(&self) -> anyhow::Result<Vec<Prefecture>>;
}
