use chrono::NaiveDate;
use reqwest::Method;
use rust_expense_tracker::models::report::{
    CategoryMonthTotal, CategoryTotal, MonthTotal, TotalReport,
};

use crate::http::{self, CliError};

pub async fn total(from: Option<NaiveDate>, to: Option<NaiveDate>) -> Result<(), CliError> {
    let path = format!("/reports/total{}", build_query(from, to));
    let report: TotalReport = http::request(Method::GET, &path, true, None).await?;

    println!("{}", format_total(&report));
    Ok(())
}

pub async fn by_category(from: Option<NaiveDate>, to: Option<NaiveDate>) -> Result<(), CliError> {
    let path = format!("/reports/totals_by_category{}", build_query(from, to));
    let totals: Vec<CategoryTotal> = http::request(Method::GET, &path, true, None).await?;

    for total in &totals {
        println!("{}", format_category_total(total));
    }
    Ok(())
}

pub async fn by_month(from: Option<NaiveDate>, to: Option<NaiveDate>) -> Result<(), CliError> {
    let path = format!("/reports/totals_by_month{}", build_query(from, to));
    let totals: Vec<MonthTotal> = http::request(Method::GET, &path, true, None).await?;

    for total in &totals {
        println!("{}", format_month_total(total));
    }
    Ok(())
}

pub async fn by_category_month(
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
) -> Result<(), CliError> {
    let path = format!("/reports/totals_by_category_month{}", build_query(from, to));
    let totals: Vec<CategoryMonthTotal> = http::request(Method::GET, &path, true, None).await?;

    for total in &totals {
        println!("{}", format_category_month_total(total));
    }
    Ok(())
}

fn format_total(report: &TotalReport) -> String {
    format!("Total {}", report.total)
}

fn format_category_total(total: &CategoryTotal) -> String {
    format!("{}: {}", total.category_name, total.total)
}

fn format_month_total(total: &MonthTotal) -> String {
    format!("{}: {}", total.month, total.total)
}

fn format_category_month_total(total: &CategoryMonthTotal) -> String {
    format!("{} ({}): {}", total.category_name, total.month, total.total)
}

fn build_query(from: Option<NaiveDate>, to: Option<NaiveDate>) -> String {
    let mut params = Vec::new();
    if let Some(from) = from {
        params.push(format!("from={from}"));
    }
    if let Some(to) = to {
        params.push(format!("to={to}"));
    }
    if params.is_empty() {
        String::new()
    } else {
        format!("?{}", params.join("&"))
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;

    #[test]
    fn format_total_includes_amount() {
        let report = TotalReport { total: 52.5 };
        assert_eq!(format_total(&report), "Total 52.5");
    }

    #[test]
    fn format_category_total_includes_name_and_amount() {
        let total = CategoryTotal {
            category_id: Uuid::nil(),
            category_name: "Groceries".to_string(),
            total: 20.0,
        };
        assert_eq!(format_category_total(&total), "Groceries: 20");
    }

    #[test]
    fn format_month_total_includes_month_and_amount() {
        let total = MonthTotal {
            month: "2026-06".to_string(),
            total: 15.0,
        };
        assert_eq!(format_month_total(&total), "2026-06: 15");
    }

    #[test]
    fn format_category_month_total_includes_name_month_and_amount() {
        let total = CategoryMonthTotal {
            category_id: Uuid::nil(),
            category_name: "Groceries".to_string(),
            month: "2026-06".to_string(),
            total: 15.0,
        };
        assert_eq!(format_category_month_total(&total), "Groceries (2026-06): 15");
    }

    #[test]
    fn build_query_with_no_dates_is_empty() {
        assert_eq!(build_query(None, None), "");
    }

    #[test]
    fn build_query_with_from_only() {
        let from = NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
        assert_eq!(build_query(Some(from), None), "?from=2026-07-01");
    }

    #[test]
    fn build_query_with_to_only() {
        let to = NaiveDate::from_ymd_opt(2026, 7, 31).unwrap();
        assert_eq!(build_query(None, Some(to)), "?to=2026-07-31");
    }

    #[test]
    fn build_query_with_both_dates() {
        let from = NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
        let to = NaiveDate::from_ymd_opt(2026, 7, 31).unwrap();
        assert_eq!(
            build_query(Some(from), Some(to)),
            "?from=2026-07-01&to=2026-07-31"
        );
    }
}
