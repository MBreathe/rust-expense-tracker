use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize)]
pub struct TotalReport {
    pub total: f64,
}

#[derive(Serialize)]
pub struct CategoryTotal {
    pub category_id: Uuid,
    pub category_name: String,
    pub total: f64,
}

#[derive(Serialize)]
pub struct MonthTotal {
    pub month: String,
    pub total: f64,
}

#[derive(Serialize)]
pub struct CategoryMonthTotal {
    pub category_id: Uuid,
    pub category_name: String,
    pub month: String,
    pub total: f64,
}

#[derive(Deserialize)]
pub struct ReportFilter {
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
}
