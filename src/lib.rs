use std::str::FromStr;

use axum::{
    Router,
    routing::{delete, get},
};
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

pub mod error;
pub mod handlers;
pub mod models;
pub mod state;

use handlers::{
    category::{create_category, delete_category, list_categories},
    expense::{create_expense, delete_expense, get_expense, list_expenses, update_expense},
};
use state::AppState;

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/expenses", get(list_expenses).post(create_expense))
        .route(
            "/expenses/{id}",
            get(get_expense).put(update_expense).delete(delete_expense),
        )
        .route("/categories", get(list_categories).post(create_category))
        .route("/categories/{id}", delete(delete_category))
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
