use sqlx::SqlitePool;

pub mod category;
pub mod expense;
pub mod report;

#[derive(Clone)]
pub struct AppState {
    pool: SqlitePool,
}

impl AppState {
    pub fn new(pool: SqlitePool) -> Self {
        AppState { pool }
    }
}
