use reqwest::Method;
use rust_expense_tracker::models::user::{TokenResponse, User};
use serde_json::json;

use crate::{
    http::{self, CliError},
    token,
};

pub async fn register(username: String, password: String) -> Result<(), CliError> {
    let User { id, username } = http::request(
        Method::POST,
        "/auth/register",
        false,
        Some(json!({"username": username, "password": password})),
    )
    .await?;

    println!("Created user {username} (id: {id})");
    Ok(())
}

pub async fn login(username: String, password: String) -> Result<(), CliError> {
    let TokenResponse { token: token_value } = http::request(
        Method::POST,
        "/auth/login",
        false,
        Some(json!({"username": username, "password": password})),
    )
    .await?;

    token::save_token(&token_value)?;

    println!("Logged in as {username}");
    Ok(())
}
