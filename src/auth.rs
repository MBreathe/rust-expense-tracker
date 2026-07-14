use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{error::AppError, state::AppState};

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: Uuid,
    exp: usize,
}

pub struct AuthUser(pub Uuid);

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::Unauthorized("missing authorization header"))?;

        let token = header
            .strip_prefix("Bearer ")
            .ok_or(AppError::Unauthorized("invalid authorization header"))?;
        let claims = decode::<Claims>(
            token,
            &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| AppError::Unauthorized("invalid or expired token"))?
        .claims;
        Ok(AuthUser(claims.sub))
    }
}

pub fn issue_token(user_id: Uuid, secret: &str) -> String {
    let expiry = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after the unix epoch")
        .as_secs() as usize
        + 24 * 60 * 60; // 24 hrs

    let claims = Claims {
        sub: user_id,
        exp: expiry,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .expect("encoding JWT should not fail")
}
