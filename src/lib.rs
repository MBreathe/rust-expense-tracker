use std::str::FromStr;

use axum::Router;
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

pub mod auth;
pub mod error;
pub mod handlers;
pub mod models;
pub mod state;

use handlers::{category, expense, report};
use state::AppState;

use crate::handlers::user;

pub fn app(state: AppState) -> Router {
    Router::new()
        .nest("/categories", category::routes())
        .nest("/expenses", expense::routes())
        .nest("/reports", report::routes())
        .nest("/auth", user::routes())
        .with_state(state)
}

pub async fn connect(database_url: &str) -> SqlitePool {
    let connect_options = SqliteConnectOptions::from_str(database_url)
        .unwrap()
        .foreign_keys(true);
    let pool = SqlitePoolOptions::new()
        .connect_with(connect_options)
        .await
        .unwrap();
    sqlx::migrate!().run(&pool).await.unwrap();
    pool
}
