use chrono::NaiveDate;
use reqwest::Method;
use rust_expense_tracker::models::expense::Expense;
use serde_json::json;
use uuid::Uuid;

use crate::http::{self, CliError};

pub async fn list() -> Result<(), CliError> {
    let expenses: Vec<Expense> = http::request(Method::GET, "/expenses", true, None).await?;

    for expense in &expenses {
        println!("{}", format_expense(expense));
    }
    Ok(())
}

pub async fn get(id: Uuid) -> Result<(), CliError> {
    let expense: Expense =
        http::request(Method::GET, &format!("/expenses/{id}"), true, None).await?;

    println!("{}", format_expense(&expense));
    Ok(())
}

pub async fn create(
    amount: f64,
    category_id: Uuid,
    description: String,
    date: NaiveDate,
) -> Result<(), CliError> {
    let expense: Expense = http::request(Method::POST, "/expenses", true, Some(json!({"amount": amount, "category_id": category_id, "description": description, "date": date}))).await?;

    println!("{}", format_expense(&expense));
    Ok(())
}

pub async fn update(
    id: Uuid,
    amount: f64,
    category_id: Uuid,
    description: String,
    date: NaiveDate,
) -> Result<(), CliError> {
    let expense: Expense = http::request(Method::PUT, &format!("/expenses/{id}"), true, Some(json!({"amount": amount, "category_id": category_id, "description": description, "date": date}))).await?;

    println!("{}", format_expense(&expense));
    Ok(())
}

pub async fn delete(id: Uuid) -> Result<(), CliError> {
    let Expense { description, .. } =
        http::request(Method::DELETE, &format!("/expenses/{id}"), true, None).await?;

    println!("Deleted expense {description}");
    Ok(())
}

fn format_expense(expense: &Expense) -> String {
    let Expense {
        id,
        amount,
        category_id,
        description,
        date,
    } = expense;
    format!("{id}  {amount}  {category_id}  {description}  {date}")
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;

    #[test]
    fn format_expense_includes_all_fields() {
        let expense = Expense {
            id: Uuid::nil(),
            amount: 12.5,
            category_id: Uuid::nil(),
            description: "milk".to_string(),
            date: NaiveDate::from_ymd_opt(2026, 7, 9).unwrap(),
        };

        assert_eq!(
            format_expense(&expense),
            "00000000-0000-0000-0000-000000000000  12.5  00000000-0000-0000-0000-000000000000  milk  2026-07-09"
        );
    }
}
