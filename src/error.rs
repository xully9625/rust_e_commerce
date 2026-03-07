use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error; // I recommend adding 'thiserror' to Cargo.toml

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("Not found")]
    NotFound,

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Bad request: {0}")]
    #[allow(dead_code)]
    BadRequest(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::SqlxError(e) => {
                eprintln!("DATABASE ERROR: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An internal database error occurred".to_string(),
                )
            }
            AppError::NotFound => (StatusCode::NOT_FOUND, "Resource not found".to_string()),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
