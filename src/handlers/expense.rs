use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::{
    error::AppError,
    models::expense::{Expense, NewExpense},
    state::AppState,
};

pub async fn list_expenses(State(state): State<AppState>) -> Result<Json<Vec<Expense>>, AppError> {
    let expenses = state.list_expenses().await?;
    Ok(Json(expenses))
}

pub async fn create_expense(
    State(state): State<AppState>,
    Json(new_expense): Json<NewExpense>,
) -> Result<Json<Expense>, AppError> {
    let expense = Expense {
        id: Uuid::new_v4(),
        amount: new_expense.amount,
        category_id: new_expense.category_id,
        description: new_expense.description,
        date: new_expense.date,
    };
    state.insert_expense(expense.clone()).await?;
    Ok(Json(expense))
}

pub async fn get_expense(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Expense>, AppError> {
    let expense = state
        .get_expense(id)
        .await?
        .ok_or(AppError::NotFound("expense"))?;
    Ok(Json(expense))
}

pub async fn update_expense(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(new_expense): Json<NewExpense>,
) -> Result<Json<Expense>, AppError> {
    state
        .get_expense(id)
        .await?
        .ok_or(AppError::NotFound("expense"))?;
    let expense = Expense {
        id,
        amount: new_expense.amount,
        category_id: new_expense.category_id,
        description: new_expense.description,
        date: new_expense.date,
    };
    state.update_expense(expense.clone()).await?;
    Ok(Json(expense))
}

pub async fn delete_expense(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Expense>, AppError> {
    let expense = state
        .remove_expense(id)
        .await?
        .ok_or(AppError::NotFound("expense"))?;
    Ok(Json(expense))
}
