use crate::{models::expense::Expense, state::AppState};
use chrono::NaiveDate;
use uuid::Uuid;

impl AppState {
    pub async fn list_expenses(&self) -> Result<Vec<Expense>, sqlx::Error> {
        sqlx::query_as!(
            Expense,
            r#"SELECT id as "id: Uuid", amount, category_id as "category_id: Uuid", description, date as "date: NaiveDate" FROM expenses"#)
            .fetch_all(&self.pool)
            .await
    }
    pub async fn insert_expense(&self, expense: Expense) -> Result<(), sqlx::Error> {
        sqlx::query_as!(
            Expense,
            "INSERT INTO expenses (id, amount, category_id, description, date) VALUES (?, ?, ?, ?, ?)",
            expense.id,
            expense.amount,
            expense.category_id,
            expense.description,
            expense.date)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
    pub async fn update_expense(&self, expense: Expense) -> Result<(), sqlx::Error> {
        sqlx::query!(
                "UPDATE expenses SET amount = ?, category_id = ?, description = ?, date = ? WHERE id = ?",
                expense.amount,
                expense.category_id,
                expense.description,
                expense.date,
                expense.id,
            )
            .execute(&self.pool)
            .await?;
        Ok(())
    }
    pub async fn get_expense(&self, id: Uuid) -> Result<Option<Expense>, sqlx::Error> {
        sqlx::query_as!(
            Expense,
            r#"SELECT id as "id: Uuid", amount, category_id as "category_id: Uuid", description, date as "date: NaiveDate" FROM expenses WHERE id = ?"#,
            id)
            .fetch_optional(&self.pool)
            .await
    }
    pub async fn remove_expense(&self, id: Uuid) -> Result<Option<Expense>, sqlx::Error> {
        let expense = self.get_expense(id).await?;
        sqlx::query_as!(
            Expense,
            r#"DELETE FROM expenses WHERE id = ? RETURNING id as "id: Uuid", amount, category_id as "category_id: Uuid", description, date as "date: NaiveDate""#,
            id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(expense)
    }
}
