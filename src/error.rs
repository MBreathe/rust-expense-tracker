use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub enum AppError {
    NotFound(&'static str),
    Internal(sqlx::Error),
    Conflict(&'static str),
    Unauthorized(&'static str),
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        if let Some(db_err) = err.as_database_error() {
            match db_err.kind() {
                sqlx::error::ErrorKind::ForeignKeyViolation => {
                    return AppError::Conflict(
                        "category is referenced by an expense, or does not exist",
                    );
                }
                sqlx::error::ErrorKind::UniqueViolation => {
                    return AppError::Conflict("username already taken");
                }
                _ => {}
            }
        }
        AppError::Internal(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::NotFound(resource) => {
                (StatusCode::NOT_FOUND, format!("{resource} not found")).into_response()
            }
            AppError::Internal(err) => {
                (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()
            }
            AppError::Conflict(err) => (StatusCode::CONFLICT, err).into_response(),
            AppError::Unauthorized(err) => (StatusCode::UNAUTHORIZED, err).into_response(),
        }
    }
}
