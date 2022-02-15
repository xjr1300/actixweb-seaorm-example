use std::sync::Arc;

use sea_orm::ConnectionTrait;

use domains::models::common::Prefecture;

use crate::database_service::DatabaseService;

/// 都道府県のリストを返却する。
///
/// # Arguments
///
/// * `repos` - リポジトリエクステンション。
///
/// # Returns
///
/// `Result`。返却される`Result`の内容は以下の通り。
///
/// * `Ok`: 都道府県のリスト。
/// * `Err`: エラー。
pub async fn list(repos: Arc<dyn DatabaseService>) -> anyhow::Result<Vec<Prefecture>> {
    let txn = repos.connection().begin().await?;
    let result = repos.prefecture(&txn).list().await?;
    txn.commit().await?;

    Ok(result)
}

/// 指定された都道府県コードと一致する都道府県を検索して返却する。
///
/// # Arguments
///
/// * `repos` - リポジトリエクステンション。
/// * `code` - 都道府県コード。
///
/// # Returns
///
/// * `Ok`: 都道府県。検索できなかった場合は`None`。
/// * `Err`: エラー。
pub async fn find_by_code(
    repos: Arc<dyn DatabaseService>,
    code: u8,
) -> anyhow::Result<Option<Prefecture>> {
    let txn = repos.connection().begin().await?;
    let result = repos.prefecture(&txn).find_by_code(code).await?;
    txn.commit().await?;

    Ok(result)
}
