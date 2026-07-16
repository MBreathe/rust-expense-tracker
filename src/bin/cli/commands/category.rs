use reqwest::Method;
use rust_expense_tracker::models::category::Category;
use serde_json::json;
use uuid::Uuid;

use crate::http::{self, CliError};

pub async fn list() -> Result<(), CliError> {
    let categories: Vec<Category> = http::request(Method::GET, "/categories", true, None).await?;

    for category in &categories {
        println!("{}", format_category(category));
    }
    Ok(())
}

pub async fn create(name: String) -> Result<(), CliError> {
    let category: Category = http::request(
        Method::POST,
        "/categories",
        true,
        Some(json!({"name": name})),
    )
    .await?;

    println!("{}", format_category(&category));
    Ok(())
}

pub async fn delete(id: Uuid) -> Result<(), CliError> {
    let Category { name, .. } =
        http::request(Method::DELETE, &format!("/categories/{id}"), true, None).await?;

    println!("Deleted category {name}");
    Ok(())
}

fn format_category(category: &Category) -> String {
    let Category { id, name } = category;
    format!("{id} {name}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_category_includes_id_and_name() {
        let category = Category {
            id: Uuid::nil(),
            name: "Groceries".to_string(),
        };

        assert_eq!(
            format_category(&category),
            "00000000-0000-0000-0000-000000000000 Groceries"
        );
    }
}
