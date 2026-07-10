use uuid::Uuid;

use crate::{models::category::Category, state::AppState};

impl AppState {
    pub async fn list_category(&self) -> Result<Vec<Category>, sqlx::Error> {
        sqlx::query_as!(Category, r#"SELECT id as "id: Uuid", name FROM categories"#)
            .fetch_all(&self.pool)
            .await
    }
    pub async fn insert_category(&self, category: Category) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO categories (id, name) VALUES (?, ?)",
            category.id,
            category.name
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    pub async fn remove_category(&self, id: Uuid) -> Result<Option<Category>, sqlx::Error> {
        sqlx::query_as!(
            Category,
            r#"DELETE FROM categories WHERE id = ? RETURNING id as "id: Uuid", name"#,
            id
        )
        .fetch_optional(&self.pool)
        .await
    }
}
