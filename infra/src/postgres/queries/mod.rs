use async_trait::async_trait;

use derive_new::new;
use sea_orm::{
    prelude::DateTimeWithTimeZone, ColumnTrait, DatabaseTransaction, EntityTrait, FromQueryResult,
    JoinType, QueryFilter, QuerySelect, RelationTrait,
};

use super::schema::prelude::Accounts;
use super::schema::{accounts, jwt_tokens, prefectures};
use domains::models::{
    accounts::{
        optional_phone_number, Account, AccountId, AccountName, FixedMobileNumbers, HashedPassword,
    },
    auth::{JwtToken, JwtTokenWithExpiredAt, JwtTokens, JwtTokensId},
    common::{Address, AddressDetails, EmailAddress, PostalCode, Prefecture},
};
use usecases::queries::{AccountQueryService, AccountTokens};

#[derive(new)]
pub struct PgAccountQueryService<'a> {
    txn: &'a DatabaseTransaction,
}

#[derive(Debug, FromQueryResult)]
struct SelectResult {
    id: String,
    email: String,
    name: String,
    password: String,
    is_active: bool,
    fixed_number: Option<String>,
    mobile_number: Option<String>,
    postal_code: String,
    prefecture_code: i16,
    address_details: String,
    logged_in_at: Option<DateTimeWithTimeZone>,
    created_at: DateTimeWithTimeZone,
    updated_at: DateTimeWithTimeZone,
    prefecture_name: String,
    tokens_id: String,
    access: Option<String>,
    access_expired_at: Option<DateTimeWithTimeZone>,
    refresh: Option<String>,
    refresh_expired_at: Option<DateTimeWithTimeZone>,
}

#[async_trait]
impl AccountQueryService for PgAccountQueryService<'_> {
    async fn find_active_account_by_id(
        &self,
        id: AccountId,
    ) -> anyhow::Result<Option<AccountTokens>> {
        let select = Accounts::find()
            .join(JoinType::InnerJoin, accounts::Relation::Prefectures.def())
            .join(JoinType::LeftJoin, accounts::Relation::JwtTokens.def())
            .column_as(prefectures::Column::Name, "prefecture_name")
            .column_as(jwt_tokens::Column::Id, "tokens_id")
            .column(jwt_tokens::Column::Access)
            .column(jwt_tokens::Column::AccessExpiredAt)
            .column(jwt_tokens::Column::Refresh)
            .column(jwt_tokens::Column::RefreshExpiredAt)
            .filter(accounts::Column::Id.eq(id.value.to_string()));
        let result = select.into_model::<SelectResult>().one(self.txn).await?;
        if result.is_none() {
            return Ok(None);
        }
        let result = result.unwrap();
        let account_id = AccountId::try_from(result.id.clone()).unwrap();
        let phone_numbers = FixedMobileNumbers::new(
            optional_phone_number(result.fixed_number.as_deref()).unwrap(),
            optional_phone_number(result.mobile_number.as_deref()).unwrap(),
        )
        .unwrap();
        let prefecture = Prefecture::new(result.prefecture_code as u8, &result.prefecture_name);
        let address_details = AddressDetails::new(&result.address_details).unwrap();
        let account = Account::new_unchecked(
            account_id.clone(),
            EmailAddress::new(&result.email).unwrap(),
            AccountName::new(&result.name).unwrap(),
            HashedPassword::new_unchecked(&result.password),
            result.is_active,
            phone_numbers,
            PostalCode::new(&result.postal_code).unwrap(),
            Address::new(prefecture, address_details),
            result.logged_in_at,
            result.created_at,
            result.updated_at,
        );
        let mut tokens: Option<JwtTokens> = None;
        if result.access.is_some() {
            let tokens_id = JwtTokensId::try_from(result.tokens_id.clone()).unwrap();
            let access = JwtTokenWithExpiredAt {
                token: JwtToken::new(&result.access.unwrap()).unwrap(),
                expired_at: result.access_expired_at.unwrap(),
            };
            let refresh = JwtTokenWithExpiredAt {
                token: JwtToken::new(&result.refresh.unwrap()).unwrap(),
                expired_at: result.refresh_expired_at.unwrap(),
            };
            tokens = Some(JwtTokens::new(tokens_id, account_id, access, refresh));
        }

        Ok(Some(AccountTokens { account, tokens }))
    }
}
