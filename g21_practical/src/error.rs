use actix_web::{HttpResponse, ResponseError};
use derive_more::Display;
use sqlx::Error as SqlxError;

#[derive(Debug, Display)]
pub enum AppError {
    #[display(fmt = "Database error: {}", _0)]
    Database(SqlxError),

    #[display(fmt = "Not found: {}", _0)]
    NotFound(String),

    // BadRequest variant for client input errors:
    #[display(fmt = "Bad Request: {}", _0)]
    BadRequest(String),

    #[display(fmt = "Unauthorized: {}", _0)]
    Unauthorized(String),
}

impl std::error::Error for AppError {} // Implements the standard Error trait

// Tells actix how to convert this error into an HTTP response
impl ResponseError for AppError {
    // When you return Result<T, AppError>, this function is auto called by actix
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::Database(_) => HttpResponse::InternalServerError().body("Database error occurred"),
            AppError::NotFound(msg) => HttpResponse::NotFound().body(msg.clone()),
            AppError::BadRequest(msg) => HttpResponse::BadRequest().body(msg.clone()),
            AppError::Unauthorized(msg) => HttpResponse::Unauthorized().body(msg.clone()),
        }
    }
}