use axum::{Json, Router, extract::State, routing::post};

use crate::{
    auth,
    error::AppError,
    models::user::{Credentials, TokenResponse, User},
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
}

pub async fn register(
    State(state): State<AppState>,
    Json(credentials): Json<Credentials>,
) -> Result<Json<User>, AppError> {
    let user = state.insert_user(credentials).await?;
    Ok(Json(user))
}

pub async fn login(
    State(state): State<AppState>,
    Json(credentials): Json<Credentials>,
) -> Result<Json<TokenResponse>, AppError> {
    let user_id = state
        .verify_credentials(credentials)
        .await?
        .ok_or(AppError::Unauthorized("invalid username or password"))?;
    let token = auth::issue_token(user_id, &state.jwt_secret);
    Ok(Json(TokenResponse { token }))
}
