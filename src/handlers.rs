use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{Expense, NewExpense},
    state::AppState,
};

pub async fn list_expenses(State(state): State<AppState>) -> Json<Vec<Expense>> {
    Json(state.list())
}

pub async fn create_expense(
    State(state): State<AppState>,
    Json(new_expense): Json<NewExpense>,
) -> Json<Expense> {
    let expense = Expense {
        id: Uuid::new_v4(),
        amount: new_expense.amount,
        category: new_expense.category,
        description: new_expense.description,
        date: new_expense.date,
    };
    state.insert(expense.clone());
    Json(expense)
}

pub async fn get_expense(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Expense>, AppError> {
    let expense = state.get(id).ok_or(AppError::NotFound)?;
    Ok(Json(expense))
}

pub async fn update_expense(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(new_expense): Json<NewExpense>,
) -> Result<Json<Expense>, AppError> {
    state.get(id).ok_or(AppError::NotFound)?;
    let expense = Expense {
        id,
        amount: new_expense.amount,
        category: new_expense.category,
        description: new_expense.description,
        date: new_expense.date,
    };
    state.insert(expense.clone());
    Ok(Json(expense))
}

pub async fn delete_expense(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Expense>, AppError> {
    let expense = state.remove(id).ok_or(AppError::NotFound)?;
    Ok(Json(expense))
}
