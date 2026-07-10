use std::env::var;

use rust_expense_tracker::{app, connect, state::AppState};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = connect(&database_url).await;
    let state = AppState::new(pool);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app(state)).await.unwrap();
}
