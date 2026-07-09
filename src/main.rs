use axum::{Router, routing::get};
use tokio::net::TcpListener;

use crate::{
    handlers::{create_expense, delete_expense, get_expense, list_expenses, update_expense},
    state::AppState,
};

mod error;
mod handlers;
mod models;
mod state;

#[tokio::main]
async fn main() {
    let state = AppState::new();

    let app = Router::new()
        .route("/expenses", get(list_expenses).post(create_expense))
        .route(
            "/expenses/{id}",
            get(get_expense).put(update_expense).delete(delete_expense),
        )
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
