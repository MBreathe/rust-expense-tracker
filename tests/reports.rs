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
    AppState::new(pool, "test-secret".to_string())
}

async fn request(
    app: &Router,
    method: &str,
    uri: &str,
    token: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(token) = token {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
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

/// Registers and logs in a fresh test user, returning a bearer token to use
/// against every protected route in these tests.
async fn register_and_login(app: &Router) -> String {
    request(
        app,
        "POST",
        "/auth/register",
        None,
        Some(json!({"username": "testuser", "password": "hunter2"})),
    )
    .await;
    let (_, login) = request(
        app,
        "POST",
        "/auth/login",
        None,
        Some(json!({"username": "testuser", "password": "hunter2"})),
    )
    .await;
    login["token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn total_report_on_empty_db_returns_zero() {
    let app = app(test_state().await);
    let token = register_and_login(&app).await;

    let (status, report) = request(&app, "GET", "/reports/total", Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(report["total"], 0.0);
}

#[tokio::test]
async fn total_report_sums_all_expenses() {
    let app = app(test_state().await);
    let token = register_and_login(&app).await;

    let (_, category) = request(
        &app,
        "POST",
        "/categories",
        Some(&token),
        Some(json!({"name": "Groceries"})),
    )
    .await;
    let category_id = category["id"].as_str().unwrap();

    request(
        &app,
        "POST",
        "/expenses",
        Some(&token),
        Some(json!({"amount": 12.5, "category_id": category_id, "description": "milk", "date": "2026-07-09"})),
    )
    .await;
    request(
        &app,
        "POST",
        "/expenses",
        Some(&token),
        Some(json!({"amount": 40.0, "category_id": category_id, "description": "rice", "date": "2026-07-10"})),
    )
    .await;

    let (status, report) = request(&app, "GET", "/reports/total", Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(report["total"], 52.5);
}

#[tokio::test]
async fn total_report_respects_date_range() {
    let app = app(test_state().await);
    let token = register_and_login(&app).await;

    let (_, category) = request(
        &app,
        "POST",
        "/categories",
        Some(&token),
        Some(json!({"name": "Groceries"})),
    )
    .await;
    let category_id = category["id"].as_str().unwrap();

    request(
        &app,
        "POST",
        "/expenses",
        Some(&token),
        Some(json!({"amount": 12.5, "category_id": category_id, "description": "milk", "date": "2026-06-15"})),
    )
    .await;
    request(
        &app,
        "POST",
        "/expenses",
        Some(&token),
        Some(json!({"amount": 40.0, "category_id": category_id, "description": "rice", "date": "2026-07-10"})),
    )
    .await;

    let (status, report) = request(
        &app,
        "GET",
        "/reports/total?from=2026-07-01&to=2026-07-31",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(report["total"], 40.0);
}

#[tokio::test]
async fn totals_by_category_groups_and_omits_empty_categories() {
    let app = app(test_state().await);
    let token = register_and_login(&app).await;

    let (_, groceries) = request(
        &app,
        "POST",
        "/categories",
        Some(&token),
        Some(json!({"name": "Groceries"})),
    )
    .await;
    let groceries_id = groceries["id"].as_str().unwrap();
    let (_, rent) = request(
        &app,
        "POST",
        "/categories",
        Some(&token),
        Some(json!({"name": "Rent"})),
    )
    .await;
    let rent_id = rent["id"].as_str().unwrap();
    request(
        &app,
        "POST",
        "/categories",
        Some(&token),
        Some(json!({"name": "Unused"})),
    )
    .await;

    request(
        &app,
        "POST",
        "/expenses",
        Some(&token),
        Some(json!({"amount": 12.5, "category_id": groceries_id, "description": "milk", "date": "2026-07-09"})),
    )
    .await;
    request(
        &app,
        "POST",
        "/expenses",
        Some(&token),
        Some(json!({"amount": 7.5, "category_id": groceries_id, "description": "eggs", "date": "2026-07-10"})),
    )
    .await;
    request(
        &app,
        "POST",
        "/expenses",
        Some(&token),
        Some(json!({"amount": 1000.0, "category_id": rent_id, "description": "rent", "date": "2026-07-01"})),
    )
    .await;

    let (status, totals) = request(
        &app,
        "GET",
        "/reports/totals_by_category",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let totals = totals.as_array().unwrap();
    assert_eq!(totals.len(), 2);

    let groceries_total = totals
        .iter()
        .find(|t| t["category_id"] == groceries_id)
        .unwrap();
    assert_eq!(groceries_total["total"], 20.0);

    let rent_total = totals.iter().find(|t| t["category_id"] == rent_id).unwrap();
    assert_eq!(rent_total["total"], 1000.0);
}

#[tokio::test]
async fn totals_by_month_groups_by_year_month_and_sorts() {
    let app = app(test_state().await);
    let token = register_and_login(&app).await;

    let (_, category) = request(
        &app,
        "POST",
        "/categories",
        Some(&token),
        Some(json!({"name": "Groceries"})),
    )
    .await;
    let category_id = category["id"].as_str().unwrap();

    request(
        &app,
        "POST",
        "/expenses",
        Some(&token),
        Some(json!({"amount": 10.0, "category_id": category_id, "description": "june", "date": "2026-06-15"})),
    )
    .await;
    request(
        &app,
        "POST",
        "/expenses",
        Some(&token),
        Some(json!({"amount": 5.0, "category_id": category_id, "description": "june again", "date": "2026-06-20"})),
    )
    .await;
    request(
        &app,
        "POST",
        "/expenses",
        Some(&token),
        Some(json!({"amount": 40.0, "category_id": category_id, "description": "july", "date": "2026-07-10"})),
    )
    .await;

    let (status, totals) = request(
        &app,
        "GET",
        "/reports/totals_by_month",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let totals = totals.as_array().unwrap();
    assert_eq!(totals.len(), 2);
    assert_eq!(totals[0]["month"], "2026-06");
    assert_eq!(totals[0]["total"], 15.0);
    assert_eq!(totals[1]["month"], "2026-07");
    assert_eq!(totals[1]["total"], 40.0);
}

#[tokio::test]
async fn totals_by_category_month_groups_by_category_and_month() {
    let app = app(test_state().await);
    let token = register_and_login(&app).await;

    let (_, groceries) = request(
        &app,
        "POST",
        "/categories",
        Some(&token),
        Some(json!({"name": "Groceries"})),
    )
    .await;
    let groceries_id = groceries["id"].as_str().unwrap();
    let (_, rent) = request(
        &app,
        "POST",
        "/categories",
        Some(&token),
        Some(json!({"name": "Rent"})),
    )
    .await;
    let rent_id = rent["id"].as_str().unwrap();

    request(
        &app,
        "POST",
        "/expenses",
        Some(&token),
        Some(json!({"amount": 10.0, "category_id": groceries_id, "description": "milk", "date": "2026-06-05"})),
    )
    .await;
    request(
        &app,
        "POST",
        "/expenses",
        Some(&token),
        Some(json!({"amount": 5.0, "category_id": groceries_id, "description": "eggs", "date": "2026-06-20"})),
    )
    .await;
    request(
        &app,
        "POST",
        "/expenses",
        Some(&token),
        Some(json!({"amount": 8.0, "category_id": groceries_id, "description": "bread", "date": "2026-07-02"})),
    )
    .await;
    request(
        &app,
        "POST",
        "/expenses",
        Some(&token),
        Some(json!({"amount": 1000.0, "category_id": rent_id, "description": "rent", "date": "2026-06-01"})),
    )
    .await;

    let (status, totals) = request(
        &app,
        "GET",
        "/reports/totals_by_category_month",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let totals = totals.as_array().unwrap();
    assert_eq!(totals.len(), 3);

    let groceries_june = totals
        .iter()
        .find(|t| t["category_id"] == groceries_id && t["month"] == "2026-06")
        .unwrap();
    assert_eq!(groceries_june["total"], 15.0);

    let groceries_july = totals
        .iter()
        .find(|t| t["category_id"] == groceries_id && t["month"] == "2026-07")
        .unwrap();
    assert_eq!(groceries_july["total"], 8.0);

    let rent_june = totals
        .iter()
        .find(|t| t["category_id"] == rent_id && t["month"] == "2026-06")
        .unwrap();
    assert_eq!(rent_june["total"], 1000.0);
}

#[tokio::test]
async fn reports_with_inverted_date_range_return_empty() {
    let app = app(test_state().await);
    let token = register_and_login(&app).await;

    let (_, category) = request(
        &app,
        "POST",
        "/categories",
        Some(&token),
        Some(json!({"name": "Groceries"})),
    )
    .await;
    let category_id = category["id"].as_str().unwrap();
    request(
        &app,
        "POST",
        "/expenses",
        Some(&token),
        Some(json!({"amount": 10.0, "category_id": category_id, "description": "milk", "date": "2026-07-09"})),
    )
    .await;

    let (status, total) = request(
        &app,
        "GET",
        "/reports/total?from=2026-08-01&to=2026-07-01",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(total["total"], 0.0);

    let (status, by_category) = request(
        &app,
        "GET",
        "/reports/totals_by_category?from=2026-08-01&to=2026-07-01",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(by_category.as_array().unwrap().len(), 0);
}
