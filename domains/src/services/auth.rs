use std::str::FromStr;

use common::ENV_VALUES;

use super::super::models::accounts::{Account, RawPassword};
use super::super::models::common::EmailAddress;
use super::super::repositories::accounts::AccountRepository;
use super::hashers::{decode_password, gen_hashed_password, PasswordHashFunc};

/// パスワードを検証する。
///
/// # Arguments
///
/// * `raw_password` - ハッシュ化していないパスワード。
/// * `hashed_password` - データベースに記録しているパスワード。ハッシュ化アルゴリズム、ハッシュ化ラウンド数、ソルト文字数、ソルト、ハッシュ化したパスワード。
///
/// # Returns
///
/// `Result`。返却された`Result`の内容は以下の通り。
///
/// * `Ok`: パスワードの検証に成功した場合はtrue。パスワードの検証に失敗した場合はfalse。
/// * `Err`: エラー。
pub fn verify_password(raw_password: &str, hashed_password: &str) -> anyhow::Result<bool> {
    // ハッシュ化されたパスワードをデコード
    let (algo, round, _, sault, hashed) = decode_password(hashed_password)?;
    let func = PasswordHashFunc::from_str(&algo)?;
    // 検証するパスワードをハッシュ化
    let target = gen_hashed_password(
        raw_password,
        &sault,
        &ENV_VALUES.password_pepper,
        func,
        round,
    );

    // ハッシュ化されたパスワードを確認
    Ok(target == hashed)
}

/// ユーザーを認証する。
///
/// # Arguments
///
/// * `repo` - アカウントリポジトリ。
/// * `email` - ユーザーのアカウントに登録したEメールアドレス。
/// * `password` - ユーザーのアカウントに登録したパスワード。
///
/// # Returns
///
/// `Result`。返却された`Result`の内容は以下の通り。
///
/// * `Ok`: 認証に成功した場合はアカウント。認証に失敗した場合は`None`。
/// * `Err`: エラー。
pub async fn authenticate(
    repo: &dyn AccountRepository,
    email: EmailAddress,
    password: RawPassword,
) -> anyhow::Result<Option<Account>> {
    // Eメールアドレスでアカウントを検索
    let result = repo.find_by_email(email).await?;
    if result.is_none() {
        // アカウントが見つからなかった場合
        return Ok(None);
    }
    let account = result.unwrap();
    // アカウントがアクティブでない場合は認証に失敗
    if !account.is_active() {
        return Ok(None);
    }
    // パスワードを検証
    if !verify_password(&password.value(), &account.password().value())? {
        return Ok(None);
    }

    Ok(Some(account))
}
