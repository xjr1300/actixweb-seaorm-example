use std::future::Future;
use std::pin::Pin;

use actix_web::{error::ErrorUnauthorized, Error, FromRequest};

use common::jwt_token::{decode_jwt_token, Claims};

/// JWT認証列挙体
///
/// HTTPリクエストヘッダの`Authorization`に記録されている`Bearer`トークンで
/// 認証済みであるかを示す。
#[derive(Clone)]
pub enum JwtAuth {
    /// 認証状態(データにクレーム)を管理
    Authenticate(Claims),
    /// 認証されていない
    Anonymous,
}

impl FromRequest for JwtAuth {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_http::Payload,
    ) -> Self::Future {
        // Authorizationヘッダを取得
        let auth = req.headers().get("Authorization");
        if auth.is_none() {
            return Box::pin(async move { Ok(JwtAuth::Anonymous) });
        }
        let auth = auth.unwrap().to_owned();
        // Bearerトークンを取得
        let split: Vec<&str> = auth.to_str().unwrap().split("Bearer").collect();
        let token = split[1].trim().to_owned();
        // トークンをデコード
        Box::pin(async move {
            decode_jwt_token(&token)
                .map(JwtAuth::Authenticate)
                .map_err(|err| ErrorUnauthorized(format!("{}", err)))
        })
    }
}
