use sea_orm::{DatabaseConnection, DatabaseTransaction};

use domains::repositories::{
    accounts::AccountRepository, auth::JwtTokensRepository, common::PrefectureRepository,
};

use crate::queries::AccountQueryService;

/// データベースサービス
pub trait DatabaseService: Send + Sync {
    /// データベースコネクションを返却する。
    ///
    /// # Returns
    ///
    /// データベースコネクション。
    fn connection(&self) -> DatabaseConnection;

    /// 都道府県リポジトリを返却する。
    ///
    /// # Returns
    ///
    fn prefecture<'a>(&self, txn: &'a DatabaseTransaction) -> Box<dyn PrefectureRepository + 'a>;

    /// アカウントリポジトリを返却する。
    ///
    /// # Returns
    ///
    /// アカウントリポジトリ。
    fn account<'a>(&self, txn: &'a DatabaseTransaction) -> Box<dyn AccountRepository + 'a>;

    /// JWTトークンリポジトリを返却する。
    ///
    /// # Returns
    ///
    /// JWTトークンリポジトリ。
    fn jwt_tokens<'a>(&self, txn: &'a DatabaseTransaction) -> Box<dyn JwtTokensRepository + 'a>;

    /// アカウントクエリサービスを変革する。
    ///
    /// # Returns
    ///
    /// アカウントクエリサービス。
    fn account_service<'a>(
        &self,
        txn: &'a DatabaseTransaction,
    ) -> Box<dyn AccountQueryService + 'a>;
}
