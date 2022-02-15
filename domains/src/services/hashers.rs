use std::str::FromStr;

use anyhow::anyhow;
use sha2::{Digest, Sha224, Sha256, Sha384, Sha512, Sha512_224, Sha512_256};
use strum_macros::{Display, EnumIter, EnumString};

use common::ENV_VALUES;

#[cfg(test)]
use mockall;

/// パスワードハッシュ関数列挙型。
#[derive(Debug, PartialEq, Clone, Copy, Display, EnumString, EnumIter)]
pub enum PasswordHashFunc {
    /// SHA-224ハッシュ関数。
    #[strum(serialize = "SHA-224")]
    SHA224,
    /// SHA-256ハッシュ関数。
    #[strum(serialize = "SHA-256")]
    SHA256,
    /// SHA-387ハッシュ関数。
    #[strum(serialize = "SHA-384")]
    SHA384,
    /// SHA-512ハッシュ関数。
    #[strum(serialize = "SHA-512")]
    SHA512,
    /// SHA-512/224ハッシュ関数。
    #[strum(serialize = "SHA-512/224")]
    SHA512_224,
    /// SHA-512/256ハッシュ関数。
    #[strum(serialize = "SHA-512/256")]
    SHA512_256,
}

/// 環境変数からパスワードをハッシュ化するハッシュ関数の種類を取得する。
///
/// # Returns
///
/// * ハッシュ関数の種類を示す`PasswordHashFunc`列挙型の値。
/// * 環境変数からハッシュ関数の種類を得られなかった場合は`Error`列挙体の値。
fn password_hash_func() -> anyhow::Result<PasswordHashFunc> {
    match PasswordHashFunc::from_str(&ENV_VALUES.password_hash_func) {
        Ok(hash_func) => Ok(hash_func),
        _ => Err(anyhow!(
            "パスワードをハッシュ化する関数を指定する環境変数PASSWORD_HASH_FUNCの値が不正です。"
        )),
    }
}

/// ソルトに使用する文字を連結した文字列。
const SAULT_CHARS: &str = r##"!"#$%&'()*-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\]^_`abcdefghijklmnopqrstuvwxyz{|}~"##;

/// ソルトが生成する機能を提供する構造体が実装するトレイト。
#[cfg_attr(test, mockall::automock)]
pub trait SaultProvider {
    fn generate(&self, len: usize) -> String;
}

/// ソルトを生成する構造体。
pub struct SaultProviderImpl;

impl SaultProvider for SaultProviderImpl {
    /// ソルトを生成する。
    ///
    /// # Arguments
    ///
    /// * `len` - 生成するソルトの長さ。
    ///
    /// # Returns
    ///
    /// * ソルト。
    fn generate(&self, len: usize) -> String {
        let chars: Vec<char> = SAULT_CHARS.chars().collect();
        let mut result = String::with_capacity(len);
        unsafe {
            for _ in 0..len {
                result.push(*chars.get_unchecked(fastrand::usize(0..chars.len())));
            }
        }

        result
    }
}

/// 文字列をハッシュ化した結果を文字列で返却する。
///
/// # Arguments
///
/// * `func` - 文字列をハッシュ化するハッシュ関数を示す列挙型。
/// * `target` - ハッシュ化する文字列。
///
/// # Returns
///
/// * ハッシュ化した文字列。
fn hash_func_doit(func: PasswordHashFunc, target: &str) -> String {
    match func {
        PasswordHashFunc::SHA224 => {
            let mut hasher = Sha224::new();
            hasher.update(target);
            hex::encode(hasher.finalize().to_vec())
        }
        PasswordHashFunc::SHA256 => {
            let mut hasher = Sha256::new();
            hasher.update(target);
            hex::encode(hasher.finalize().to_vec())
        }
        PasswordHashFunc::SHA384 => {
            let mut hasher = Sha384::new();
            hasher.update(target);
            hex::encode(hasher.finalize().to_vec())
        }
        PasswordHashFunc::SHA512 => {
            let mut hasher = Sha512::new();
            hasher.update(target);
            hex::encode(hasher.finalize().to_vec())
        }
        PasswordHashFunc::SHA512_224 => {
            let mut hasher = Sha512_224::new();
            hasher.update(target);
            hex::encode(hasher.finalize().to_vec())
        }
        PasswordHashFunc::SHA512_256 => {
            let mut hasher = Sha512_256::new();
            hasher.update(target);
            hex::encode(hasher.finalize().to_vec())
        }
    }
}

/// パスワードにソルトとペッパーを加えた文字列をハッシュ化したパスワードを返却する。
///
/// # Arguments
///
/// * `raw` - ハッシュ化するパスワード。
/// * `sault` - パスワードに追加するソルト。
/// * `pepper` - パスワードに追加するソルト。
/// * `func` - パスワードをハッシュ化する関数。
/// * `round` - パスワードをハッシュ化するラウンド数。
///
/// # Returns
///
/// ハッシュ化したパスワード。
pub fn gen_hashed_password(
    raw: &str,
    sault: &str,
    pepper: &str,
    func: PasswordHashFunc,
    round: u32,
) -> String {
    let mut hashed = format!("{}{}{}", raw, sault, pepper);
    for _ in 0..round {
        hashed = hash_func_doit(func, &hashed);
    }

    hashed
}

/// パスワードにソルトとペッパーを加えた文字列をハッシュ化した文字列を返却する。
///
/// パスワードにソルトとペッパーを加えた文字列をハッシュ化した文字列を返却する。
/// 返却する文字列は下記の通り生成される。また、対応するハッシュ関数を以下に示す。
/// ハッシュ関数は環境変数`PASSWORD_HASH_FUNC`から判別して、環境変数`PASSWORD_HASH_FUNC`は、
/// 下に示した文字列を設定する。
///
/// * SHA-224
/// * SHA-256
/// * SHA-384
/// * SHA-512
/// * SHA-512/224
/// * SHA-512/256
///
/// 1. 環境変数からハッシュ関数(PASSWORD_HASH_FUNC)、ソルトの長さ(PASSWORD_SAULT)、
///    ペッパー(PASSWORD_PEPPER)及びラウンド回数(PASSWORD_HASH_ROUND)を取得する。
/// 2. ソルトとなる文字列を生成する。
/// 3. パスワードの末尾にソルト、ペッパーの順に文字列を追加した文字列を生成する。
/// 5. 上記文字列をラウンド回数だけハッシュ関数でハッシュ化した文字列を生成する。
/// 6. ハッシュ関数名$ラウンド回数$ソルト$ハッシュ化文字列の書式で文字列を返却する。
///
/// # Arguments
///
/// * `raw` - ハッシュ化する前のパスワード（生パスワード）。
///
/// # Returns
///
/// * ハッシュアルゴリズム、ラウンド回数、ソルト及びパスワードにソルトとペッパーを加えた文字列を指定された回数だけハッシュ化した文字列を
///   `$`で連結した文字列。返却される文字列の書式は、`<algo>$<round>$<sault_len>$<sault>$<hashed>`。
pub fn hash_password(sault_provider: &dyn SaultProvider, raw: &str) -> anyhow::Result<String> {
    let func = password_hash_func()?;
    // パスワードの末尾にソルトとペッパーを追加して、ハッシュ化対象文字列を生成
    let sault = sault_provider.generate(ENV_VALUES.password_sault_len);
    let hashed = gen_hashed_password(
        raw,
        &sault,
        &ENV_VALUES.password_pepper,
        func,
        ENV_VALUES.password_hash_round,
    );

    Ok(format!(
        "{}${}${}${}${}",
        func, ENV_VALUES.password_hash_round, ENV_VALUES.password_sault_len, sault, hashed
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ソルトを正常に生成できることを確認する。
    #[test]
    fn test_generate_sault() {
        let generator = SaultProviderImpl {};
        for len in 1..=100 {
            let sault = generator.generate(len);
            assert_eq!(sault.len(), len, "{}", sault);
            for ch in sault.chars() {
                let index = SAULT_CHARS.find(ch);
                if index.is_none() {
                    assert!(
                        false,
                        "生成したソルトにソルトに使用できない文字が含まれています。"
                    );
                }
            }
        }
    }
}

/// ハッシュ化されたパスワードをデコードする。
///
/// # Arguments
///
/// * `password` - ハッシュ化されたパスワード。
///
/// # Returns
///
/// `Result`。返却される`Result`の内容は以下の通り。
///
/// * `Ok`: アルゴリズム、ハッシュ化ラウンド数、ソルト文字数、ソルト、パスワードをハッシュ化した結果を格納したタプル。
/// * `Err`: エラー。
pub fn decode_password(password: &str) -> anyhow::Result<(String, u32, usize, String, String)> {
    // アルゴリズムの記録の終了を示す`$`の位置を検索
    let algo_pos = password.find('$');
    if algo_pos.is_none() {
        return Err(anyhow!(
            "ハッシュ化したパスワードから、アルゴリズムを取得できません。 "
        ));
    };
    let algo_pos = algo_pos.unwrap();
    let algo = &password[..algo_pos];
    let mut start = algo_pos + 1;
    // ラウンド数の記録の終了を示す'$'の位置を検索
    let round_pos = password[start..].find('$');
    if round_pos.is_none() {
        return Err(anyhow!(
            "ハッシュ化したパスワードから、ハッシュ化ラウンド数を取得できません。 "
        ));
    };
    let round_pos = round_pos.unwrap();
    let round = &password[start..start + round_pos];
    let round = round.parse::<u32>();
    if round.is_err() {
        return Err(anyhow!(
            "ハッシュ化したパスワードから取得したハッシュ化ラウンド数を数値に変換できません。"
        ));
    }
    start += round_pos + 1;
    // ソルトの文字数の記録の終了を示す'$'の位置を検索
    let len_pos = password[start..].find('$');
    if len_pos.is_none() {
        return Err(anyhow!(
            "ハッシュ化したパスワードから、ソルトの文字数を取得できません。 "
        ));
    }
    let len_pos = len_pos.unwrap();
    let len = &password[start..start + len_pos];
    let len = len.parse::<usize>();
    if len.is_err() {
        return Err(anyhow!(
            "ハッシュ化したパスワードから取得したソルトの文字数を数値に変換できません。"
        ));
    }
    let len = len.unwrap();
    start += len_pos + 1;
    // ソルトを取得
    let sault = &password[start..start + len];
    start += len + 1;
    // パスワードをハッシュ化した結果を取得
    let hashed = &password[start..];

    Ok((
        algo.to_owned(),
        round.unwrap(),
        len,
        sault.to_owned(),
        hashed.to_owned(),
    ))
}

#[cfg(test)]
mod decode_password_test {
    use super::*;

    // ハッシュ化したパスワードをデコードできることを確認する。
    #[test]
    fn test_decode_password() {
        // <algo>$<round>$<sault>$<hashed>
        let algo = "SHA256";
        let round: u32 = 10;
        let sault = "this-is-sault";
        let len = sault.len();
        let hashed = "this-is-hashed-password";
        let password = format!("{}${}${}${}${}", algo, round, len, sault, hashed);
        let result = decode_password(&password);
        assert!(result.is_ok());
        assert_eq!(result.as_ref().unwrap().0, algo);
        assert_eq!(result.as_ref().unwrap().1, round);
        assert_eq!(result.as_ref().unwrap().2, len);
        assert_eq!(result.as_ref().unwrap().3, sault);
        assert_eq!(result.as_ref().unwrap().4, hashed);
    }
}
