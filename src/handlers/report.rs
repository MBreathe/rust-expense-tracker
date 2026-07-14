use axum::{
    Json, Router,
    extract::{Query, State},
    routing::get,
};

use crate::{
    error::AppError,
    models::report::{CategoryMonthTotal, CategoryTotal, MonthTotal, ReportFilter, TotalReport},
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/total", get(total_report))
        .route("/totals_by_category", get(totals_by_category))
        .route("/totals_by_month", get(totals_by_month))
        .route("/totals_by_category_month", get(totals_by_category_month))
}

pub async fn total_report(
    State(state): State<AppState>,
    Query(filter): Query<ReportFilter>,
) -> Result<Json<TotalReport>, AppError> {
    let report = state.total_report(filter.from, filter.to).await?;
    Ok(Json(report))
}

pub async fn totals_by_category(
    State(state): State<AppState>,
    Query(filter): Query<ReportFilter>,
) -> Result<Json<Vec<CategoryTotal>>, AppError> {
    let totals = state.totals_by_category(filter.from, filter.to).await?;
    Ok(Json(totals))
}

pub async fn totals_by_month(
    State(state): State<AppState>,
    Query(filter): Query<ReportFilter>,
) -> Result<Json<Vec<MonthTotal>>, AppError> {
    let totals = state.totals_by_month(filter.from, filter.to).await?;
    Ok(Json(totals))
}

pub async fn totals_by_category_month(
    State(state): State<AppState>,
    Query(filter): Query<ReportFilter>,
) -> Result<Json<Vec<CategoryMonthTotal>>, AppError> {
    let totals = state
        .totals_by_category_month(filter.from, filter.to)
        .await?;
    Ok(Json(totals))
}
