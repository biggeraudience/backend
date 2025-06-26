use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    DbError(#[from] sqlx::Error),
    #[error("JWT error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
    #[error("Auth error: {0}")]
    AuthError(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Resource not found: {0}")]
    NotFound(String),
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Forbidden")]
    Forbidden,
    #[error("File upload error: {0}")]
    FileUploadError(String),
    #[error("Serialization/Deserialization error: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("Multipart error: {0}")]
    MultipartError(#[from] actix_multipart::MultipartError),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Generic error: {0}")]
    GenericError(String),
    #[error("Internal server error")]
    InternalServerError,
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        use tracing::error; // Use tracing for logging errors

        let (status_code, error_message) = match self {
            AppError::DbError(e) => {
                error!("Database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "A database error occurred.".to_string())
            },
            AppError::JwtError(e) => {
                error!("JWT error: {:?}", e);
                (StatusCode::UNAUTHORIZED, "Invalid or expired token.".to_string())
            },
            AppError::AuthError(msg) => {
                (StatusCode::BAD_REQUEST, msg.clone())
            },
            AppError::ValidationError(msg) => {
                (StatusCode::BAD_REQUEST, msg.clone())
            },
            AppError::NotFound(resource) => {
                (StatusCode::NOT_FOUND, format!("{} not found.", resource))
            },
            AppError::Unauthorized => {
                (StatusCode::UNAUTHORIZED, "Authentication required.".to_string())
            },
            AppError::Forbidden => {
                (StatusCode::FORBIDDEN, "Access denied.".to_string())
            },
            AppError::FileUploadError(e) => {
                error!("File upload error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "File upload failed.".to_string())
            },
            AppError::SerdeError(e) => {
                error!("Serialization/Deserialization error: {:?}", e);
                (StatusCode::BAD_REQUEST, "Invalid data format.".to_string())
            },
            AppError::MultipartError(e) => {
                error!("Multipart error: {:?}", e);
                (StatusCode::BAD_REQUEST, "Invalid multipart data.".to_string())
            },
            AppError::IoError(e) => {
                error!("IO error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "An I/O error occurred.".to_string())
            },
            AppError::GenericError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, msg.clone())
            },
            AppError::InternalServerError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "An unexpected error occurred.".to_string())
            },
        };

        HttpResponse::build(status_code)
            .json(json!({"error": error_message}))
    }
}
