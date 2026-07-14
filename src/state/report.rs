use std::collections::HashMap;

use chrono::NaiveDate;
use uuid::Uuid;

use crate::{
    models::report::{CategoryMonthTotal, CategoryTotal, MonthTotal, TotalReport},
    state::AppState,
};

impl AppState {
    pub async fn total_report(
        &self,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
    ) -> Result<TotalReport, sqlx::Error> {
        let row = sqlx::query!(
            r#"SELECT COALESCE(SUM(amount), 0.0) as "total!: f64"
            FROM expenses
            WHERE (? IS NULL OR date >= ?) AND (? IS NULL OR date <= ?)"#,
            from,
            from,
            to,
            to
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(TotalReport { total: row.total })
    }
    pub async fn totals_by_category(
        &self,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
    ) -> Result<Vec<CategoryTotal>, sqlx::Error> {
        sqlx::query_as!(
            CategoryTotal,
            r#"SELECT c.id as "category_id: Uuid", c.name as "category_name!: String", SUM(e.amount) as "total!: f64"
            FROM expenses e
            JOIN categories c ON c.id = e.category_id
            WHERE (? IS NULL OR e.date >= ?) AND (? IS NULL OR e.date <= ?)
            GROUP BY c.id, c.name"#,
            from,
            from,
            to,
            to
        )
        .fetch_all(&self.pool)
        .await
    }
    pub async fn totals_by_month(
        &self,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
    ) -> Result<Vec<MonthTotal>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"SELECT date as "date: NaiveDate", amount
            FROM expenses
            WHERE (? IS NULL OR date >= ?) AND (? IS NULL OR date <= ?)"#,
            from,
            from,
            to,
            to
        )
        .fetch_all(&self.pool)
        .await?;

        let mut totals: HashMap<String, f64> = HashMap::new();
        for row in rows {
            let month = row.date.format("%Y-%m").to_string();
            *totals.entry(month).or_insert(0.0) += row.amount;
        }

        let mut result: Vec<MonthTotal> = totals
            .into_iter()
            .map(|(month, total)| MonthTotal { month, total })
            .collect();
        result.sort_by(|a, b| a.month.cmp(&b.month));
        Ok(result)
    }
    pub async fn totals_by_category_month(
        &self,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
    ) -> Result<Vec<CategoryMonthTotal>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"SELECT c.id as "category_id: Uuid", c.name as "category_name!: String", e.date as "date: NaiveDate", e.amount
            FROM expenses e
            JOIN categories c ON c.id = e.category_id
            WHERE (? IS NULL OR e.date >= ?) AND (? IS NULL OR e.date <= ?)"#,
            from,
            from,
            to,
            to
        )
        .fetch_all(&self.pool)
        .await?;

        let mut totals: HashMap<(Uuid, String), (String, f64)> = HashMap::new();
        for row in rows {
            let month = row.date.format("%Y-%m").to_string();
            let entry = totals
                .entry((row.category_id, month))
                .or_insert_with(|| (row.category_name.clone(), 0.0));
            entry.1 += row.amount;
        }

        let mut result: Vec<CategoryMonthTotal> = totals
            .into_iter()
            .map(
                |((category_id, month), (category_name, total))| CategoryMonthTotal {
                    category_id,
                    category_name,
                    month,
                    total,
                },
            )
            .collect();
        result.sort_by(|a, b| (&a.category_name, &a.month).cmp(&(&b.category_name, &b.month)));
        Ok(result)
    }
}
