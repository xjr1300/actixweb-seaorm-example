use std::marker::PhantomData;

use derive_new::new;
use sea_orm::DatabaseTransaction;

/// PostgreSQLリポジトリ構造体
#[derive(new)]
pub struct PgRepository<'a, T> {
    /// データベースコネクション。
    pub txn: &'a DatabaseTransaction,
    /// マーカー。
    _marker: PhantomData<T>,
}
