use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use jsonwebtoken::{errors::ErrorKind, Algorithm, DecodingKey, EncodingKey, Header};
use serde_json::Value;
use tracing::error;

const SANTA_RSA_KEY: &[u8] = include_bytes!("day16_santa_public_key.pem");

pub fn router() -> axum::Router {
    axum::Router::new()
        .route("/wrap", post(wrap))
        .route("/unwrap", get(unwrap))
        .route("/decode", post(decode))
}

async fn wrap(jar: CookieJar, Json(body): Json<Value>) -> CookieJar {
    let jwt = jsonwebtoken::encode(
        &Header::default(),
        &body,
        &EncodingKey::from_secret("cch24".as_ref()),
    )
    .expect("jwt token creation must succeed");
    jar.add(Cookie::new("gift", jwt))
}

async fn unwrap(jar: CookieJar) -> Response {
    let Some(gift) = jar.get("gift") else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let jwt = gift.value();

    let mut jwt_validation = jsonwebtoken::Validation::default();
    jwt_validation.required_spec_claims = Default::default();
    jwt_validation.validate_exp = false;
    let Ok(value) = jsonwebtoken::decode::<Value>(
        jwt,
        &DecodingKey::from_secret("cch24".as_ref()),
        &jwt_validation,
    ) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    value.claims.to_string().into_response()
}

async fn decode(jwt: String) -> Response {
    let mut jwt_validation = jsonwebtoken::Validation::default();
    jwt_validation.algorithms = vec![Algorithm::RS256, Algorithm::RS512];
    jwt_validation.required_spec_claims = Default::default();
    jwt_validation.validate_exp = false;
    let decode = jsonwebtoken::decode::<Value>(
        &jwt,
        &DecodingKey::from_rsa_pem(SANTA_RSA_KEY).unwrap(),
        &jwt_validation,
    );
    match decode {
        Ok(value) => value.claims.to_string().into_response(),
        Err(err) if *err.kind() == ErrorKind::InvalidSignature => {
            StatusCode::UNAUTHORIZED.into_response()
        }
        Err(err) => {
            error!(%err, "jwt decode failed");
            StatusCode::BAD_REQUEST.into_response()
        }
    }
}
