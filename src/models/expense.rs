use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct Expense {
    pub id: Uuid,
    pub amount: f64,
    pub category_id: Uuid,
    pub description: String,
    pub date: NaiveDate,
}

#[derive(Deserialize)]
pub struct NewExpense {
    pub amount: f64,
    pub category_id: Uuid,
    pub description: String,
    pub date: NaiveDate,
}
