use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Clone)]
pub struct User {
    pub id: Uuid,
    pub username: String,
}

#[derive(Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct TokenResponse {
    pub token: String,
}
