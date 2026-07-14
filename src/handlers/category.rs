use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{delete, get},
};
use uuid::Uuid;

use crate::{
    error::AppError,
    models::category::{Category, NewCategory},
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_categories).post(create_category))
        .route("/{id}", delete(delete_category))
}

pub async fn list_categories(
    State(state): State<AppState>,
) -> Result<Json<Vec<Category>>, AppError> {
    let categories = state.list_category().await?;
    Ok(Json(categories))
}
pub async fn create_category(
    State(state): State<AppState>,
    Json(new_category): Json<NewCategory>,
) -> Result<Json<Category>, AppError> {
    let category = Category {
        id: Uuid::new_v4(),
        name: new_category.name,
    };
    state.insert_category(category.clone()).await?;
    Ok(Json(category))
}
pub async fn delete_category(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Category>, AppError> {
    let category = state
        .remove_category(id)
        .await?
        .ok_or(AppError::NotFound("category"))?;
    Ok(Json(category))
}
