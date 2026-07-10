use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct Category {
    pub id: Uuid,
    pub name: String,
}

#[derive(Deserialize)]
pub struct NewCategory {
    pub name: String,
}
