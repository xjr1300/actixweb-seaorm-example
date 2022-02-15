use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

use domains::{
    models::{
        accounts::AccountId,
        auth::{self, JwtToken, JwtTokenWithExpiredAt, JwtTokensId},
    },
    repositories::auth::JwtTokensRepository,
};

use crate::postgres::schema::jwt_tokens;

use super::super::schema::jwt_tokens::{ActiveModel, Column, Entity, Model};
use super::super::schema::prelude::JwtTokens;
use super::common::PgRepository;

/// 有効期限付きアクセス・リフレッシュトークンリポジトリ型
pub type PgJwtTokensRepository<'a> = PgRepository<'a, auth::JwtTokens>;

fn model_to_active_model(tokens: &auth::JwtTokens) -> ActiveModel {
    ActiveModel {
        id: Set(tokens.id().value.to_string()),
        account_id: Set(tokens.account_id().value.to_string()),
        access: Set(tokens.access().token.value()),
        access_expired_at: Set(tokens.access().expired_at),
        refresh: Set(tokens.refresh().token.value()),
        refresh_expired_at: Set(tokens.refresh().expired_at),
    }
}

fn db_to_model(db: &Model) -> auth::JwtTokens {
    let access = JwtTokenWithExpiredAt {
        token: JwtToken::new(&db.access).unwrap(),
        expired_at: db.access_expired_at,
    };
    let refresh = JwtTokenWithExpiredAt {
        token: JwtToken::new(&db.refresh).unwrap(),
        expired_at: db.refresh_expired_at,
    };
    auth::JwtTokens::new(
        JwtTokensId::try_from(db.id.as_str()).unwrap(),
        AccountId::try_from(db.account_id.as_str()).unwrap(),
        access,
        refresh,
    )
}

#[async_trait]
impl JwtTokensRepository for PgJwtTokensRepository<'_> {
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
    async fn find_by_id(&self, id: JwtTokensId) -> anyhow::Result<Option<auth::JwtTokens>> {
        let result = JwtTokens::find_by_id(id.value.to_string())
            .one(self.txn)
            .await?;
        if result.is_none() {
            return Ok(None);
        }

        Ok(Some(db_to_model(&result.unwrap())))
    }

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
    async fn find_by_access_token(&self, token: &str) -> anyhow::Result<Option<auth::JwtTokens>> {
        let result = JwtTokens::find()
            .filter(jwt_tokens::Column::Access.eq(token))
            .one(self.txn)
            .await?;
        if result.is_none() {
            return Ok(None);
        }

        Ok(Some(db_to_model(&result.unwrap())))
    }

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
    async fn find_by_refresh_token(&self, token: &str) -> anyhow::Result<Option<auth::JwtTokens>> {
        let result = JwtTokens::find()
            .filter(jwt_tokens::Column::Refresh.eq(token))
            .one(self.txn)
            .await?;
        if result.is_none() {
            return Ok(None);
        }

        Ok(Some(db_to_model(&result.unwrap())))
    }

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
    async fn insert(&self, tokens: &auth::JwtTokens) -> anyhow::Result<auth::JwtTokens> {
        let active_model = model_to_active_model(tokens);
        let _ = active_model.insert(self.txn).await?;

        Ok(self.find_by_id(tokens.id()).await?.unwrap())
    }

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
    async fn delete(&self, id: AccountId) -> anyhow::Result<()> {
        let _ = Entity::delete_many()
            .filter(Column::Id.eq(id.value.to_string()))
            .exec(self.txn)
            .await?;

        Ok(())
    }
}
