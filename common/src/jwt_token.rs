use anyhow::anyhow;
use chrono::{TimeZone, Utc};
use hmac::{Hmac, Mac};
use jwt::{Header, SignWithKey, Token, VerifyWithKey};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use crate::ENV_VALUES;

/// クレイム
#[derive(Default, Deserialize, Serialize)]
pub struct Claims {
    /// アカウントID.
    pub sub: String,
    /// 有効期限を示すUnixエポック(1970-01-01(UTC)からの経過秒数)。
    pub exp: i64,
}

/// JWTトークンを生成する。
///
/// # Arguments
///
/// * `claims` - クレイム。
///
/// # Returns
///
/// `Result`。返却される`Result`の内容は以下の通り。
///
/// * `Ok`: JWT。
/// * `Err`: エラー。
pub fn gen_jwt_token(claims: &Claims) -> anyhow::Result<String> {
    // 環境変数から秘密鍵を取得して鍵を生成
    let secret_key = &ENV_VALUES.jwt_token_secret_key;
    let key: Hmac<Sha256> = Hmac::new_from_slice(secret_key.as_bytes())
        .map_err(|err| anyhow!("トークを生成する鍵の生成に失敗しました。{}", err))?;
    // JWTを生成
    let header: Header = Default::default();
    let unsigned_token = Token::new(header, claims);
    let signed_token = unsigned_token
        .sign_with_key(&key)
        .map_err(|err| anyhow!("トークンの生成に失敗しました。{}", err))?;

    Ok(signed_token.into())
}

/// JWTトークンをデコードする。
///
/// # Arguments
///
/// * `token` - JWTトークン。
///
/// # Returns
///
/// `Result`。返却される`Result`の内容は以下の通り。
///
/// * `Ok`: アカウントIDを示す文字列と、トークンの有効期限を示すUnixエポック(1970-01-01からの経過秒数)。
/// * `Err`: エラー。
pub fn decode_jwt_token(token: &str) -> anyhow::Result<Claims> {
    // 環境変数から秘密鍵を取得して鍵を生成
    let secret_key = &ENV_VALUES.jwt_token_secret_key;
    let key: Hmac<Sha256> = Hmac::new_from_slice(secret_key.as_bytes())
        .map_err(|err| anyhow!("トークンを生成する鍵の生成に失敗しました。{}", err))?;
    // トークンをデコード
    let token: Token<Header, Claims, _> = VerifyWithKey::verify_with_key(token, &key)
        .map_err(|err| anyhow!("トークンのデコードに失敗しました。{}", err))?;
    let (_, claims) = token.into();
    // トークンの有効期限を確認
    let expired = Utc.timestamp(claims.exp, 0);
    if expired <= Utc::now() {
        return Err(anyhow!("トークンの有効期限が切れています。"));
    }

    Ok(claims)
}

/*
#[cfg(test)]
mod auth_tests {
    use super::*;
    use common::ENV_VALUES;
    use domains::models::accounts::AccountId;
    use dotenv;
    use jwt::VerifyWithKey;

    /// JWTを正常に生成できることを確認する。
    #[test]
    fn test_gen_jwt() {
        dotenv::from_filename(".env.dev").ok();
        // JWTを生成
        let id = Ulid::new().to_string();
        let expired = local_now(None) + Duration::days(1);
        let token = gen_jwt_token(id.clone(), expired);
        if let Err(ref err) = token {
            assert!(false, "JWTを生成できませんでした。{:?}。", err);
        }
        // 生成したトークンを検証
        let token = token.unwrap();
        let secret_key = &ENV_VALUES.jwt_token_secret_key;
        let key: Hmac<Sha256> = Hmac::new_from_slice(secret_key.as_bytes()).unwrap();
        let claims: BTreeMap<String, String> = token.verify_with_key(&key).unwrap();
        assert_eq!(claims["sub"], id.value.to_string());
        assert_eq!(claims["exp"], expired.timestamp().to_string());
    }
}
*/
