use std::{env::var, str::FromStr};

use axum::{
    Router,
    routing::{delete, get},
};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use tokio::net::TcpListener;

use crate::{
    handlers::{
        category::{create_category, delete_category, list_categories},
        expense::{create_expense, delete_expense, get_expense, list_expenses, update_expense},
    },
    state::AppState,
};

mod error;
mod handlers;
mod models;
mod state;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set");

    let connect_options = SqliteConnectOptions::from_str(&database_url)
        .unwrap()
        .foreign_keys(true);
    let pool = SqlitePoolOptions::new()
        .connect_with(connect_options)
        .await
        .unwrap();
    sqlx::migrate!().run(&pool).await.unwrap();

    let state = AppState::new(pool);

    let app = Router::new()
        .route("/expenses", get(list_expenses).post(create_expense))
        .route(
            "/expenses/{id}",
            get(get_expense).put(update_expense).delete(delete_expense),
        )
        .route("/categories", get(list_categories).post(create_category))
        .route("/categories/{id}", delete(delete_category))
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
