use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde_json::json;
use thiserror::Error;
use tracing::error;

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
    #[error("Bcrypt error: {0}")]
    BcryptError(#[from] bcrypt::BcryptError),
    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Generic error: {0}")]
    GenericError(String),
    #[error("Internal server error")]
    InternalServerError,
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let (status, msg) = match self {
            AppError::DbError(e) => {
                error!("DB error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "A database error occurred.")
            }
            AppError::JwtError(e) => {
                error!("JWT error: {:?}", e);
                (StatusCode::UNAUTHORIZED, "Invalid or expired token.")
            }
            AppError::AuthError(m)       => (StatusCode::BAD_REQUEST, m.as_str()),
            AppError::ValidationError(m) => (StatusCode::BAD_REQUEST, m.as_str()),
            AppError::NotFound(r)        => (StatusCode::NOT_FOUND, &format!("{} not found.", r)),
            AppError::Unauthorized       => (StatusCode::UNAUTHORIZED, "Authentication required."),
            AppError::Forbidden          => (StatusCode::FORBIDDEN, "Access denied."),
            AppError::FileUploadError(_) => {
                error!("File upload failed: {:?}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, "File upload failed.")
            }
            AppError::SerdeError(e)  => {
                error!("Serde error: {:?}", e);
                (StatusCode::BAD_REQUEST, "Invalid data format.")
            }
            AppError::MultipartError(e) => {
                error!("Multipart error: {:?}", e);
                (StatusCode::BAD_REQUEST, "Invalid multipart data.")
            }
            AppError::IoError(e) => {
                error!("IO error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "An I/O error occurred.")
            }
            AppError::BcryptError(e) => {
                error!("Bcrypt error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Password processing failed.")
            }
            AppError::ReqwestError(e) => {
                error!("Reqwest error: {:?}", e);
                if e.is_timeout() {
                    (StatusCode::REQUEST_TIMEOUT, "Network request timed out.")
                } else if e.is_connect() {
                    (StatusCode::SERVICE_UNAVAILABLE, "Failed to connect to external service.")
                } else {
                    (StatusCode::INTERNAL_SERVER_ERROR, "External service communication failed.")
                }
            }
            AppError::GenericError(m) => (StatusCode::INTERNAL_SERVER_ERROR, m.as_str()),
            AppError::InternalServerError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "An unexpected error occurred.")
            }
        };

        HttpResponse::build(status).json(json!({ "error": msg }))
    }
}
