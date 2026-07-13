use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use rust_expense_tracker::{app, state::AppState};
use serde_json::{Value, json};
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

/// Shared helper: send one request against a (cloned) router, and return the
/// status code plus the parsed JSON body (Value::Null if the body is empty,
/// e.g. on some error responses that return plain text instead of JSON).
async fn request(
    app: &Router,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    let body = match body {
        Some(json) => {
            builder = builder.header("content-type", "application/json");
            Body::from(json.to_string())
        }
        None => Body::empty(),
    };
    let request = builder.body(body).unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);

    (status, value)
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
    let category: Value = serde_json::from_slice(&body).unwrap();
    let category_id = category["id"].as_str().unwrap();

    let create_expense = Request::builder().method("POST").uri("/expenses").header("content-type", "application/json").body(Body::from(json!({"amount": 12.5, "category_id": category_id, "description": "milk", "date": "2026-07-09"}).to_string())).unwrap();

    let response = app.oneshot(create_expense).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn list_expenses_returns_created_expenses() {
    let app = app(test_state().await);

    let (_, category) = request(
        &app,
        "POST",
        "/categories",
        Some(json!({"name": "Groceries"})),
    )
    .await;
    let category_id = category["id"].as_str().unwrap();

    request(
        &app,
        "POST",
        "/expenses",
        Some(json!({"amount": 12.5, "category_id": category_id, "description": "milk", "date": "2026-07-09"})),
    )
    .await;
    request(
        &app,
        "POST",
        "/expenses",
        Some(json!({"amount": 40.0, "category_id": category_id, "description": "rice", "date": "2026-07-10"})),
    )
    .await;

    let (status, expenses) = request(&app, "GET", "/expenses", None).await;
    assert_eq!(status, StatusCode::OK);

    let descriptions: Vec<&str> = expenses
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["description"].as_str().unwrap())
        .collect();
    assert!(descriptions.contains(&"milk"));
    assert!(descriptions.contains(&"rice"));
}

#[tokio::test]
async fn list_categories_returns_created_categories() {
    let app = app(test_state().await);

    request(
        &app,
        "POST",
        "/categories",
        Some(json!({"name": "Groceries"})),
    )
    .await;
    request(&app, "POST", "/categories", Some(json!({"name": "Rent"}))).await;

    let (status, categories) = request(&app, "GET", "/categories", None).await;
    assert_eq!(status, StatusCode::OK);

    let names: Vec<&str> = categories
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"Groceries"));
    assert!(names.contains(&"Rent"));
}

#[tokio::test]
async fn get_expense_returns_404_for_unknown_id() {
    let app = app(test_state().await);

    let (status, _) = request(
        &app,
        "GET",
        "/expenses/00000000-0000-0000-0000-000000000000",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_category_returns_404_for_unknown_id() {
    let app = app(test_state().await);

    let (status, _) = request(
        &app,
        "DELETE",
        "/categories/00000000-0000-0000-0000-000000000000",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn create_expense_with_unknown_category_returns_409() {
    let app = app(test_state().await);

    let (status, _) = request(
        &app,
        "POST",
        "/expenses",
        Some(json!({
            "amount": 5.0,
            "category_id": "00000000-0000-0000-0000-000000000000",
            "description": "bogus",
            "date": "2026-07-09"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn delete_category_still_referenced_returns_409() {
    let app = app(test_state().await);

    let (_, category) = request(&app, "POST", "/categories", Some(json!({"name": "Rent"}))).await;
    let category_id = category["id"].as_str().unwrap();

    let (_, expense) = request(
        &app,
        "POST",
        "/expenses",
        Some(json!({"amount": 1000.0, "category_id": category_id, "description": "rent", "date": "2026-07-09"})),
    )
    .await;
    let expense_id = expense["id"].as_str().unwrap();

    let (status, _) = request(&app, "DELETE", &format!("/categories/{category_id}"), None).await;
    assert_eq!(status, StatusCode::CONFLICT);

    // the expense should be untouched by the failed delete
    let (status, _) = request(&app, "GET", &format!("/expenses/{expense_id}"), None).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn update_expense_replaces_fields() {
    let app = app(test_state().await);

    let (_, category) = request(
        &app,
        "POST",
        "/categories",
        Some(json!({"name": "Groceries"})),
    )
    .await;
    let category_id = category["id"].as_str().unwrap();

    let (_, expense) = request(
        &app,
        "POST",
        "/expenses",
        Some(json!({"amount": 10.0, "category_id": category_id, "description": "milk", "date": "2026-07-09"})),
    )
    .await;
    let expense_id = expense["id"].as_str().unwrap();

    let (status, updated) = request(
        &app,
        "PUT",
        &format!("/expenses/{expense_id}"),
        Some(json!({"amount": 25.0, "category_id": category_id, "description": "milk and eggs", "date": "2026-07-10"})),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["amount"], 25.0);
    assert_eq!(updated["description"], "milk and eggs");
    assert_eq!(updated["date"], "2026-07-10");
}

#[tokio::test]
async fn delete_expense_removes_it() {
    let app = app(test_state().await);

    let (_, category) = request(
        &app,
        "POST",
        "/categories",
        Some(json!({"name": "Groceries"})),
    )
    .await;
    let category_id = category["id"].as_str().unwrap();

    let (_, expense) = request(
        &app,
        "POST",
        "/expenses",
        Some(json!({"amount": 10.0, "category_id": category_id, "description": "milk", "date": "2026-07-09"})),
    )
    .await;
    let expense_id = expense["id"].as_str().unwrap();

    let (status, deleted) = request(&app, "DELETE", &format!("/expenses/{expense_id}"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(deleted["id"], expense_id);

    let (status, _) = request(&app, "GET", &format!("/expenses/{expense_id}"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
