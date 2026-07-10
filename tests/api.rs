use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use rust_expense_tracker::{app, state::AppState};
use serde_json::json;
use sqlx::sqlite::SqlitePoolOptions;
use tower::ServiceExt;

async fn test_state() -> AppState {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!().run(&pool).await.unwrap();
    AppState::new(pool)
}

#[tokio::test]
async fn create_expense_with_valid_category() {
    let state = test_state().await;
    let app = app(state);

    let create_category = Request::builder()
        .method("POST")
        .uri("/categories")
        .header("content-type", "application/json")
        .body(Body::from(json!({"name": "Groceries"}).to_string()))
        .unwrap();

    let response = app.clone().oneshot(create_category).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let category: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let category_id = category["id"].as_str().unwrap();

    let create_expense = Request::builder().method("POST").uri("/expenses").header("content-type", "application/json").body(Body::from(json!({"amount": 12.5, "category_id": category_id, "description": "milk", "date": "2026-07-09"}).to_string())).unwrap();

    let response = app.oneshot(create_expense).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

// TODO: list_expenses_returns_created_expenses
// Create a couple of expenses (against a valid category), GET /expenses,
// assert 200 and that the response contains both of them.

// TODO: list_categories_returns_created_categories
// Same idea as above, but for GET /categories.

// TODO: get_expense_returns_404_for_unknown_id
// GET /expenses/{random uuid that was never created} — assert 404 and
// check the body message (see AppError::NotFound's format).

// TODO: delete_category_returns_404_for_unknown_id
// DELETE /categories/{random uuid} — assert 404.

// TODO: create_expense_with_unknown_category_returns_409
// POST /expenses with a category_id that was never created via POST /categories
// — assert 409 (this exercises the ForeignKeyViolation branch in error.rs).

// TODO: delete_category_still_referenced_returns_409
// Create a category, create an expense against it, then try DELETE
// /categories/{that id} — assert 409, and that the expense is still there
// afterward (e.g. GET /expenses/{id} still returns 200).

// TODO: update_expense_replaces_fields
// Create an expense, PUT /expenses/{id} with different amount/description/date,
// assert 200 and that the response reflects the new values, not the old ones.

// TODO: delete_expense_removes_it
// Create an expense, DELETE /expenses/{id} — assert 200 (and the body is the
// deleted expense), then GET /expenses/{id} again — assert 404.
