use derive_new::new;
use sea_orm::{DatabaseConnection, DatabaseTransaction};

use domains::repositories::{
    accounts::AccountRepository, auth::JwtTokensRepository, common::PrefectureRepository,
};
use usecases::{database_service::DatabaseService, queries::AccountQueryService};

/// 具象型データベースサービス
#[derive(Clone, new)]
pub struct DatabaseServiceImpl {
    /// データベースコネクション。
    pub conn: DatabaseConnection,
}

impl DatabaseService for DatabaseServiceImpl {
    /// データベースコネクションを返却する。
    ///
    /// # Returns
    ///
    /// データベースコネクション。
    fn connection(&self) -> DatabaseConnection {
        self.conn.clone()
    }

    /// 都道府県リポジトリを返却する。
    ///
    /// # Returns
    ///
    /// 都道府県リポジトリ。
    fn prefecture<'a>(&self, txn: &'a DatabaseTransaction) -> Box<dyn PrefectureRepository + 'a> {
        use infra::postgres::repositories::prefectures::PgPrefectureRepository;

        Box::new(PgPrefectureRepository::new(txn))
    }

    /// アカウントリポジトリを返却する。
    ///
    /// # Returns
    ///
    /// アカウントリポジトリ。
    fn account<'a>(&self, txn: &'a DatabaseTransaction) -> Box<dyn AccountRepository + 'a> {
        use infra::postgres::repositories::accounts::PgAccountRepository;

        Box::new(PgAccountRepository::new(txn))
    }

    /// JWTトークンリポジトリを返却する。
    ///
    /// # Returns
    ///
    /// JWTトークンリポジトリ。
    fn jwt_tokens<'a>(&self, txn: &'a DatabaseTransaction) -> Box<dyn JwtTokensRepository + 'a> {
        use infra::postgres::repositories::auth::PgJwtTokensRepository;

        Box::new(PgJwtTokensRepository::new(txn))
    }

    /// アカウントクエリサービスを変革する。
    ///
    /// # Returns
    ///
    /// アカウントクエリサービス。
    fn account_service<'a>(
        &self,
        txn: &'a DatabaseTransaction,
    ) -> Box<dyn AccountQueryService + 'a> {
        use infra::postgres::queries::PgAccountQueryService;

        Box::new(PgAccountQueryService::new(txn))
    }
}
