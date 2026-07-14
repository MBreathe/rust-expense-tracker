use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use jsonwebtoken::{EncodingKey, Header, encode};
use rust_expense_tracker::{app, state::AppState};
use serde::Serialize;
use serde_json::{Value, json};
use sqlx::sqlite::SqlitePoolOptions;
use tower::ServiceExt;

const TEST_SECRET: &str = "test-secret";

async fn test_state() -> AppState {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!().run(&pool).await.unwrap();
    AppState::new(pool, TEST_SECRET.to_string())
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

#[tokio::test]
async fn register_creates_user() {
    let app = app(test_state().await);

    let (status, user) = request(
        &app,
        "POST",
        "/auth/register",
        None,
        Some(json!({"username": "alice", "password": "hunter2"})),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(user["username"], "alice");
    assert!(user["id"].is_string());
    assert!(user.get("password_hash").is_none());
    assert!(user.get("password").is_none());
}

#[tokio::test]
async fn register_duplicate_username_returns_409() {
    let app = app(test_state().await);

    request(
        &app,
        "POST",
        "/auth/register",
        None,
        Some(json!({"username": "alice", "password": "hunter2"})),
    )
    .await;

    let (status, _) = request(
        &app,
        "POST",
        "/auth/register",
        None,
        Some(json!({"username": "alice", "password": "different"})),
    )
    .await;

    assert_eq!(status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn login_with_valid_credentials_returns_token() {
    let app = app(test_state().await);

    request(
        &app,
        "POST",
        "/auth/register",
        None,
        Some(json!({"username": "alice", "password": "hunter2"})),
    )
    .await;

    let (status, login) = request(
        &app,
        "POST",
        "/auth/login",
        None,
        Some(json!({"username": "alice", "password": "hunter2"})),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(!login["token"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn login_with_wrong_password_returns_401() {
    let app = app(test_state().await);

    request(
        &app,
        "POST",
        "/auth/register",
        None,
        Some(json!({"username": "alice", "password": "hunter2"})),
    )
    .await;

    let (status, _) = request(
        &app,
        "POST",
        "/auth/login",
        None,
        Some(json!({"username": "alice", "password": "wrong"})),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

/// Wrong password and unknown username must be indistinguishable to the caller
/// -- compares raw response bytes (not the JSON-parsed value, which would
/// collapse both to `Value::Null` regardless of the actual message and make
/// this assertion meaningless).
#[tokio::test]
async fn login_with_unknown_username_returns_same_401_as_wrong_password() {
    let app = app(test_state().await);

    request(
        &app,
        "POST",
        "/auth/register",
        None,
        Some(json!({"username": "alice", "password": "hunter2"})),
    )
    .await;

    let wrong_password = Request::builder()
        .method("POST")
        .uri("/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({"username": "alice", "password": "wrong"}).to_string(),
        ))
        .unwrap();
    let response = app.clone().oneshot(wrong_password).await.unwrap();
    let wrong_password_status = response.status();
    let wrong_password_body = response.into_body().collect().await.unwrap().to_bytes();

    let unknown_username = Request::builder()
        .method("POST")
        .uri("/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({"username": "unknown", "password": "whatever"}).to_string(),
        ))
        .unwrap();
    let response = app.oneshot(unknown_username).await.unwrap();
    let unknown_username_status = response.status();
    let unknown_username_body = response.into_body().collect().await.unwrap().to_bytes();

    assert_eq!(wrong_password_status, StatusCode::UNAUTHORIZED);
    assert_eq!(unknown_username_status, StatusCode::UNAUTHORIZED);
    assert_eq!(wrong_password_body, unknown_username_body);
}

#[tokio::test]
async fn protected_route_without_token_returns_401() {
    let app = app(test_state().await);

    let (status, _) = request(&app, "GET", "/expenses", None, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn protected_route_with_invalid_token_returns_401() {
    let app = app(test_state().await);

    let (status, _) = request(&app, "GET", "/expenses", Some("not-a-real-token"), None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[derive(Serialize)]
struct TestClaims {
    sub: String,
    exp: usize,
}

#[tokio::test]
async fn protected_route_with_expired_token_returns_401() {
    let app = app(test_state().await);

    let expired_claims = TestClaims {
        sub: uuid::Uuid::new_v4().to_string(),
        exp: 0, // already expired (unix epoch)
    };
    let expired_token = encode(
        &Header::default(),
        &expired_claims,
        &EncodingKey::from_secret(TEST_SECRET.as_bytes()),
    )
    .unwrap();

    let (status, _) = request(&app, "GET", "/expenses", Some(&expired_token), None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn protected_route_with_valid_token_succeeds() {
    let app = app(test_state().await);

    request(
        &app,
        "POST",
        "/auth/register",
        None,
        Some(json!({"username": "alice", "password": "hunter2"})),
    )
    .await;
    let (_, login) = request(
        &app,
        "POST",
        "/auth/login",
        None,
        Some(json!({"username": "alice", "password": "hunter2"})),
    )
    .await;
    let token = login["token"].as_str().unwrap();

    let (status, _) = request(&app, "GET", "/expenses", Some(token), None).await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = request(
        &app,
        "POST",
        "/categories",
        Some(token),
        Some(json!({"name": "Groceries"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}
