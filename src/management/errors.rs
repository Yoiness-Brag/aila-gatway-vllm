use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub enum ApiError {
    DatabaseError(String), // Changed from sqlx::Error to String for serialization
    NotFound(String),
    Conflict(String),        // For duplicate entries, etc.
    ValidationError(String), // For DTO validation issues
    Unauthorized(String),    // For authentication failures
    Forbidden(String),       // For authorization failures
    RateLimitExceeded(String), // For rate limit violations
    InternalServerError(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::DatabaseError(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
            ApiError::NotFound(message) => (StatusCode::NOT_FOUND, message),
            ApiError::Conflict(message) => (StatusCode::CONFLICT, message),
            ApiError::ValidationError(message) => (StatusCode::BAD_REQUEST, message),
            ApiError::Unauthorized(message) => (StatusCode::UNAUTHORIZED, message),
            ApiError::Forbidden(message) => (StatusCode::FORBIDDEN, message),
            ApiError::RateLimitExceeded(message) => (StatusCode::TOO_MANY_REQUESTS, message),
            ApiError::InternalServerError(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
        };

        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => ApiError::NotFound("Resource not found".to_string()),
            _ => {
                eprintln!("Detailed Database Error: {err:?}");
                ApiError::DatabaseError("A database error occurred".to_string())
            }
        }
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        let detailed_error_message = format!(
            "Serde JSON Error Kind: {:?}, Message: {}",
            err.classify(),
            err
        );
        eprintln!(
            "Detailed Serde JSON Error before wrapping in ApiError: {detailed_error_message}"
        );
        ApiError::InternalServerError(format!("JSON processing error: {err}"))
    }
}
