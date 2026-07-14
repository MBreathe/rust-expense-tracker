use sqlx::SqlitePool;

pub mod category;
pub mod expense;
pub mod report;
pub mod user;

#[derive(Clone)]
pub struct AppState {
    pool: SqlitePool,
    pub jwt_secret: String,
}

impl AppState {
    pub fn new(pool: SqlitePool, jwt_secret: String) -> Self {
        AppState { pool, jwt_secret }
    }
}
