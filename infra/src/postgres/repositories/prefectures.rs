use async_trait::async_trait;
use sea_orm::{EntityTrait, QueryOrder};

use domains::models::common::Prefecture;
use domains::repositories::common::PrefectureRepository;

use super::super::schema::prefectures;
use super::super::schema::prelude::Prefectures;
use super::common::PgRepository;

/// 都道府県リポジトリ型
pub type PgPrefectureRepository<'a> = PgRepository<'a, Prefecture>;

impl From<prefectures::Model> for Prefecture {
    fn from(m: prefectures::Model) -> Self {
        Self::new(m.code as u8, &m.name)
    }
}

#[async_trait]
impl PrefectureRepository for PgPrefectureRepository<'_> {
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
    async fn find_by_code(&self, code: u8) -> anyhow::Result<Option<Prefecture>> {
        let entity = Prefectures::find_by_id(code as i16).one(self.txn).await?;

        match entity {
            Some(pref) => Ok(Some(pref.into())),
            None => Ok(None),
        }
    }

    /// 都道府県のリストを返却する。
    ///
    /// # Returns
    ///
    /// `Result`。返却される`Result`の内容は以下の通り。
    ///
    /// * `Ok`: 都道府県を格納したベクタ。
    /// * `Err`: エラーメッセージ。
    async fn list(&self) -> anyhow::Result<Vec<Prefecture>> {
        let entities = Prefectures::find()
            .order_by_asc(prefectures::Column::Code)
            .all(self.txn)
            .await?;

        Ok(entities.iter().map(|e| e.clone().into()).collect())
    }
}

#[cfg(test)]
mod pg_prefecture_repository_tests {
    use crate::postgres::schema::prefectures;
    // use crate::schema::prelude::Prefectures;
    use domains::models::common::Prefecture;
    // use sea_orm::{DatabaseBackend, EntityTrait, MockDatabase};

    fn tokyo_model() -> prefectures::Model {
        prefectures::Model {
            code: 13,
            name: "東京都".to_owned(),
        }
    }

    // fn osaka_model() -> prefectures::Model {
    //     prefectures::Model {
    //         code: 27,
    //         name: "大阪府".to_owned(),
    //     }
    // }

    /// 都道府県モデルを都道府県に変換できることを確認する。
    #[test]
    fn test_prefecture_from_model() {
        let model = tokyo_model();
        let prefecture = Prefecture::from(model);
        assert_eq!(prefecture.code(), 13);
        assert_eq!(prefecture.name(), "東京都");
    }

    // /// 都道府県コードを指定して都道府県を取得できることを確認する。
    // #[async_std::test]
    // async fn test_find_prefecture() {
    //     let db = MockDatabase::new(DatabaseBackend::Postgres)
    //         .append_query_results(vec![vec![tokyo_model()]])
    //         .into_connection();
    //     let result = Prefectures::find().one(&db).await;
    //     assert_eq!(result.unwrap(), Some(tokyo_model()));
    // }

    // /// 都道府県のリストを得られることを確認する。
    // #[async_std::test]
    // async fn test_prefecture_list() {
    //     let db = MockDatabase::new(DatabaseBackend::Postgres)
    //         .append_query_results(vec![vec![tokyo_model(), osaka_model()]])
    //         .into_connection();
    //     let result = Prefectures::find().all(&db).await;
    //     assert_eq!(result.unwrap(), vec![tokyo_model(), osaka_model()]);
    // }
}
