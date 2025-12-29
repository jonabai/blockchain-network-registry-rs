//! Error Types
//!
//! Domain-specific error types with proper HTTP status code mapping.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use thiserror::Error;

/// Domain-level errors representing business rule violations
#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Network with chain_id {0} already exists")]
    ChainIdConflict(i32),

    #[error("Invalid network state: {0}")]
    InvalidState(String),
}

/// Repository-level errors for data access failures
#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Entity not found: {0}")]
    NotFound(String),

    #[error("Data mapping error: {0}")]
    Mapping(String),
}

/// Use case-level errors for application logic failures
#[derive(Debug, Error)]
pub enum UseCaseError {
    #[error("Validation failed: {0:?}")]
    Validation(Vec<String>),

    #[error("{resource} with id '{id}' not found")]
    NotFound { resource: String, id: String },

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error(transparent)]
    Domain(#[from] DomainError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

impl UseCaseError {
    /// Get the HTTP status code for this error
    #[must_use]
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::NotFound { .. } => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::Domain(DomainError::ChainIdConflict(_)) => StatusCode::CONFLICT,
            Self::Domain(DomainError::InvalidState(_)) => StatusCode::BAD_REQUEST,
            Self::Repository(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Get the error code for this error
    #[must_use]
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::NotFound { .. } => "NOT_FOUND",
            Self::Conflict(_) => "CONFLICT",
            Self::Unauthorized(_) => "UNAUTHORIZED",
            Self::Forbidden(_) => "FORBIDDEN",
            Self::Domain(DomainError::ChainIdConflict(_)) => "CONFLICT",
            Self::Domain(DomainError::InvalidState(_)) => "INVALID_STATE",
            Self::Repository(_) => "INTERNAL_ERROR",
        }
    }
}

/// API error response for HTTP responses
#[derive(Debug, Error)]
pub enum ApiError {
    #[error(transparent)]
    UseCase(#[from] UseCaseError),

    #[error("Invalid request: {0}")]
    BadRequest(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Invalid UUID: {0}")]
    InvalidUuid(String),

    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),
}

/// Error response body structure
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    pub timestamp: String,
}

/// Error detail structure
#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Vec<FieldError>>,
}

/// Field-level error for validation errors
#[derive(Debug, Serialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message, details) = match &self {
            ApiError::UseCase(uc_error) => {
                let details = if let UseCaseError::Validation(errors) = uc_error {
                    Some(
                        errors
                            .iter()
                            .map(|e| FieldError {
                                field: "".to_string(),
                                message: e.clone(),
                            })
                            .collect(),
                    )
                } else {
                    None
                };
                (uc_error.status_code(), uc_error.error_code().to_string(), uc_error.to_string(), details)
            }
            ApiError::BadRequest(msg) => {
                (StatusCode::BAD_REQUEST, "BAD_REQUEST".to_string(), msg.clone(), None)
            }
            ApiError::Unauthorized => {
                (StatusCode::UNAUTHORIZED, "UNAUTHORIZED".to_string(), "Unauthorized".to_string(), None)
            }
            ApiError::InvalidUuid(msg) => {
                (StatusCode::BAD_REQUEST, "INVALID_UUID".to_string(), msg.clone(), None)
            }
            ApiError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR".to_string(),
                "An unexpected error occurred".to_string(),
                None,
            ),
        };

        let body = ErrorResponse {
            error: ErrorDetail {
                code,
                message,
                details,
            },
            request_id: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        (status, Json(body)).into_response()
    }
}

impl From<uuid::Error> for ApiError {
    fn from(err: uuid::Error) -> Self {
        ApiError::InvalidUuid(err.to_string())
    }
}

impl From<validator::ValidationErrors> for ApiError {
    fn from(err: validator::ValidationErrors) -> Self {
        let messages: Vec<String> = err
            .field_errors()
            .iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |e| {
                    format!(
                        "{}: {}",
                        field,
                        e.message.as_ref().map_or("invalid", |m| m.as_ref())
                    )
                })
            })
            .collect();
        ApiError::UseCase(UseCaseError::Validation(messages))
    }
}
