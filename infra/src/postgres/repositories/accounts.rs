use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

use domains::models::{
    accounts::{
        optional_phone_number, optional_phone_number_string, Account, AccountId, AccountName,
        FixedMobileNumbers, HashedPassword,
    },
    common::{Address, AddressDetails, EmailAddress, PostalCode, Prefecture},
};
use domains::repositories::accounts::AccountRepository;

use super::super::schema::{
    accounts, prefectures,
    prelude::{Accounts, Prefectures},
};
use super::common::PgRepository;

/// アカウントリポジトリ型
pub type PgAccountRepository<'a> = PgRepository<'a, Account>;

/// アカウントモデルと都道府県モデルからアカウントを構築して返却する。
///
/// # Arguments
///
/// * `account` - アカウントモデル。
/// * `prefecture` - 都道府県モデル。
///
/// # Returns
///
/// * アカウント。
fn model_to_account(account: &accounts::Model, prefecture: &prefectures::Model) -> Account {
    let phone_numbers = FixedMobileNumbers::new(
        optional_phone_number(account.fixed_number.as_deref()).unwrap(),
        optional_phone_number(account.mobile_number.as_deref()).unwrap(),
    )
    .unwrap();
    let prefecture = Prefecture::new(prefecture.code as u8, &prefecture.name);
    let address_details = AddressDetails::new(&account.address_details).unwrap();

    Account::new_unchecked(
        AccountId::try_from(account.id.as_str()).unwrap(),
        EmailAddress::new(&account.email).unwrap(),
        AccountName::new(&account.name).unwrap(),
        HashedPassword::from_repository(&account.password),
        account.is_active,
        phone_numbers,
        PostalCode::new(&account.postal_code).unwrap(),
        Address::new(prefecture, address_details),
        account.logged_in_at,
        account.created_at,
        account.updated_at,
    )
}

/// アカウントをアクティブモデルに変換する。
///
/// # Arguments
///
/// * `account` - アカウント。
///
/// # Returns
///
/// * アカウントのアクティブモデル。
fn account_to_active_model(account: &Account) -> accounts::ActiveModel {
    accounts::ActiveModel {
        id: Set(account.id().value.to_string()),
        email: Set(account.email().value()),
        name: Set(account.name().value()),
        password: Set(account.password().value()),
        is_active: Set(account.is_active()),
        fixed_number: Set(optional_phone_number_string(
            account.phone_numbers().fixed(),
        )),
        mobile_number: Set(optional_phone_number_string(
            account.phone_numbers().mobile(),
        )),
        postal_code: Set(account.postal_code().value()),
        prefecture_code: Set(account.address().prefecture().code() as i16),
        address_details: Set(account.address().details().value()),
        logged_in_at: Set(account.logged_in_at()),
        created_at: Set(account.created_at()),
        updated_at: Set(account.updated_at()),
    }
}

#[cfg(test)]
mod account_model_tests {
    use super::*;
    use domains::models::common::{local_now, PhoneNumber};
    use sea_orm::ActiveValue;
    use ulid::Ulid;

    /// アカウントモデルと都道府県モデルから、アカウントを構築できることを確認する。
    #[test]
    fn test_model_to_account() {
        let p = prefectures::Model {
            code: 13,
            name: String::from("東京都"),
        };
        let a = accounts::Model {
            id: Ulid::new().to_string(),
            email: String::from("taro@example.com"),
            name: String::from("taro"),
            password: String::from("this-is-hashed-password"),
            is_active: true,
            fixed_number: Some(String::from("012-345-6789")),
            mobile_number: Some(String::from("090-1234-5678")),
            postal_code: String::from("100-0014"),
            prefecture_code: p.code,
            address_details: String::from("千代田区永田町1-7-1"),
            logged_in_at: Some(local_now(None)),
            created_at: local_now(None),
            updated_at: local_now(None),
        };
        let account = model_to_account(&a, &p);
        assert_eq!(account.id().value.to_string(), a.id);
        assert_eq!(account.email().value(), a.email);
        assert_eq!(account.name().value(), a.name);
        assert_eq!(account.password().value(), a.password);
        assert_eq!(account.is_active(), a.is_active);
        assert_eq!(
            account.phone_numbers().fixed().unwrap().value(),
            a.fixed_number.unwrap()
        );
        assert_eq!(
            account.phone_numbers().mobile().unwrap().value(),
            a.mobile_number.unwrap()
        );
        assert_eq!(account.postal_code().value(), a.postal_code);
        assert_eq!(account.address().prefecture().code(), p.code as u8);
        assert_eq!(account.address().details().value(), a.address_details);
        assert_eq!(account.logged_in_at(), a.logged_in_at);
        assert_eq!(account.created_at(), a.created_at);
        assert_eq!(account.updated_at(), a.updated_at);
    }

    /// アカウントをアクティブモデルに変換できるか確認する。
    #[test]
    fn test_account_to_active_model() {
        let id = Ulid::new();
        let email = EmailAddress::new("foo@example.com").unwrap();
        let name = AccountName::new("foo").unwrap();
        let password = HashedPassword::from_repository("01abCD#$");
        let is_active = true;
        let fixed_number = PhoneNumber::new("012-345-6890").unwrap();
        let mobile_number = PhoneNumber::new("090-1234-5678").unwrap();
        let phone_numbers =
            FixedMobileNumbers::new(Some(fixed_number.clone()), Some(mobile_number.clone()))
                .unwrap();
        let postal_code = PostalCode::new("012-3456").unwrap();
        let pref_code = 13;
        let pref_name = "東京都";
        let prefecture = Prefecture::new(pref_code, pref_name);
        let address_details = AddressDetails::new("新宿区西新宿2-8-1").unwrap();
        let address = Address::new(prefecture.clone(), address_details.clone());
        let logged_in_at = Some(local_now(None));
        let created_at = local_now(None);
        let updated_at = local_now(None);
        // アカウントを構築
        let account = Account::new_unchecked(
            AccountId::new(id),
            email.clone(),
            name.clone(),
            password.clone(),
            is_active,
            phone_numbers.clone(),
            postal_code.clone(),
            address.clone(),
            logged_in_at,
            created_at,
            updated_at,
        );
        let model = account_to_active_model(&account);
        assert_eq!(model.id, ActiveValue::set(id.to_string()));
        assert_eq!(model.email, ActiveValue::set(email.value()));
        assert_eq!(model.name, ActiveValue::set(name.value()));
        assert_eq!(model.password, ActiveValue::set(password.value()));
        assert_eq!(model.is_active, ActiveValue::set(is_active));
        assert_eq!(
            model.fixed_number,
            ActiveValue::set(Some(fixed_number.value()))
        );
        assert_eq!(
            model.mobile_number,
            ActiveValue::set(Some(mobile_number.value()))
        );
        assert_eq!(model.postal_code, ActiveValue::set(postal_code.value()));
        assert_eq!(model.prefecture_code, ActiveValue::set(pref_code as i16));
        assert_eq!(
            model.address_details,
            ActiveValue::set(address_details.value())
        );
        assert_eq!(model.logged_in_at, ActiveValue::set(logged_in_at));
        assert_eq!(model.created_at, ActiveValue::set(created_at));
        assert_eq!(model.updated_at, ActiveValue::set(updated_at));
    }
}

#[async_trait]
impl AccountRepository for PgAccountRepository<'_> {
    /// アカウントIDを指定して、アカウントを検索する。
    ///
    /// # Arguments
    ///
    /// * `id` - アカウントID。
    ///
    /// # Returns
    ///
    /// `Result`。`Result`の内容を以下の通り。
    ///
    /// * `Ok`: アカウントが見つかった場合はアカウント。アカウントが見つからなかった場合は`None`。
    /// * `Err`: エラーメッセージ。
    async fn find_by_id(&self, id: AccountId) -> anyhow::Result<Option<Account>> {
        let result = Accounts::find_by_id(id.value.to_string())
            .find_also_related(Prefectures)
            .one(self.txn)
            .await?;
        if result.is_none() {
            return Ok(None);
        }
        let (account, prefecture) = result.unwrap();

        Ok(Some(model_to_account(&account, &prefecture.unwrap())))
    }

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
    async fn find_by_email(&self, email: EmailAddress) -> anyhow::Result<Option<Account>> {
        let result = Accounts::find()
            .filter(accounts::Column::Email.eq(email.value()))
            .find_also_related(Prefectures)
            .one(self.txn)
            .await?;
        if result.is_none() {
            return Ok(None);
        }
        let (account, prefecture) = result.unwrap();

        Ok(Some(model_to_account(&account, &prefecture.unwrap())))
    }

    /// アカウントのリストを返却する。
    ///
    /// # Returns
    ///
    /// `Result`。返却される`Result`の内容は以下の通り。
    ///
    /// * `Ok`: アカウントを格納したベクタ。
    /// * `Err`: エラーメッセージ。
    async fn list(&self) -> anyhow::Result<Vec<Account>> {
        let result = Accounts::find()
            .find_also_related(Prefectures)
            .all(self.txn)
            .await?;

        Ok(result
            .iter()
            .map(|(a, p)| model_to_account(a, p.as_ref().unwrap()))
            .collect())
    }

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
    /// * `Err`: エラーメッセージ。
    async fn insert(&self, account: &Account) -> anyhow::Result<Account> {
        let active_model = account_to_active_model(account);
        let _ = active_model.insert(self.txn).await?;

        Ok(self.find_by_id(account.id()).await?.unwrap())
    }

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
    /// * `Err`: エラーメッセージ。
    async fn update(&self, account: &Account) -> anyhow::Result<Account> {
        let active_model = account_to_active_model(account);
        let _ = active_model.update(self.txn).await?;

        Ok(self.find_by_id(account.id()).await?.unwrap())
    }

    /// アカウントを削除する。
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
    /// * `Err`: エラーメッセージ。
    async fn delete(&self, id: AccountId) -> anyhow::Result<()> {
        let _ = accounts::Entity::delete_many()
            .filter(accounts::Column::Id.eq(id.value.to_string()))
            .exec(self.txn)
            .await?;

        Ok(())
    }

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
    ) -> anyhow::Result<bool> {
        let result = Accounts::find_by_id(id.value.to_string())
            .one(self.txn)
            .await?;
        if result.is_none() {
            return Ok(false);
        }
        let mut active_model: accounts::ActiveModel = result.unwrap().into();
        active_model.password = Set(new_password.value());
        let _ = active_model.update(self.txn).await?;

        Ok(true)
    }
}
