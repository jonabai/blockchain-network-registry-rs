//! Error Types
//!
//! Domain-specific error types with proper HTTP status code mapping.
//!
//! Security considerations:
//! - Internal error details are logged but not exposed to clients
//! - Request IDs are included for correlation and debugging
//! - Database errors are sanitized to prevent information leakage

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

    #[error("Validation error: {0}")]
    ValidationError(String),
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

    #[error("Unique constraint violation: {0}")]
    UniqueViolation(String),
}

impl RepositoryError {
    /// Check if this is a unique constraint violation for chain_id
    #[must_use]
    pub fn is_chain_id_conflict(&self) -> bool {
        if let RepositoryError::Database(sqlx::Error::Database(db_err)) = self {
            // PostgreSQL unique violation error code is 23505
            if db_err.code().map_or(false, |c| c == "23505") {
                return db_err.message().contains("chain_id");
            }
        }
        false
    }

    /// Convert database errors to appropriate domain errors
    #[must_use]
    pub fn into_domain_error(self) -> Self {
        if self.is_chain_id_conflict() {
            return RepositoryError::UniqueViolation("chain_id already exists".to_string());
        }
        self
    }
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
            Self::Domain(DomainError::ValidationError(_)) => StatusCode::BAD_REQUEST,
            Self::Repository(RepositoryError::UniqueViolation(_)) => StatusCode::CONFLICT,
            Self::Repository(RepositoryError::NotFound(_)) => StatusCode::NOT_FOUND,
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
            Self::Domain(DomainError::ValidationError(_)) => "VALIDATION_ERROR",
            Self::Repository(RepositoryError::UniqueViolation(_)) => "CONFLICT",
            Self::Repository(RepositoryError::NotFound(_)) => "NOT_FOUND",
            Self::Repository(_) => "INTERNAL_ERROR",
        }
    }

    /// Get a safe, user-facing message (no internal details)
    #[must_use]
    pub fn safe_message(&self) -> String {
        match self {
            Self::Validation(errors) => format!("Validation failed: {}", errors.join(", ")),
            Self::NotFound { resource, id } => format!("{} with id '{}' not found", resource, id),
            Self::Conflict(msg) => msg.clone(),
            Self::Unauthorized(_) => "Authentication required".to_string(),
            Self::Forbidden(_) => "Access denied".to_string(),
            Self::Domain(DomainError::ChainIdConflict(chain_id)) => {
                format!("Network with chain_id {} already exists", chain_id)
            }
            Self::Domain(DomainError::InvalidState(msg)) => msg.clone(),
            Self::Domain(DomainError::ValidationError(msg)) => msg.clone(),
            Self::Repository(RepositoryError::UniqueViolation(field)) => {
                format!("A record with this {} already exists", field)
            }
            Self::Repository(RepositoryError::NotFound(resource)) => {
                format!("{} not found", resource)
            }
            // Don't expose internal database/mapping errors
            Self::Repository(RepositoryError::Database(_)) => {
                "An internal error occurred. Please try again later.".to_string()
            }
            Self::Repository(RepositoryError::Mapping(_)) => {
                "An internal error occurred. Please try again later.".to_string()
            }
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

impl ApiError {
    /// Check if this error should be logged at error level
    #[must_use]
    pub fn is_internal_error(&self) -> bool {
        matches!(
            self,
            ApiError::Internal(_)
                | ApiError::UseCase(UseCaseError::Repository(RepositoryError::Database(_)))
                | ApiError::UseCase(UseCaseError::Repository(RepositoryError::Mapping(_)))
        )
    }
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
        // Log internal errors for debugging (details not exposed to client)
        if self.is_internal_error() {
            tracing::error!(error = ?self, "Internal server error");
        }

        let (status, code, message, details) = match &self {
            ApiError::UseCase(uc_error) => {
                let details = if let UseCaseError::Validation(errors) = uc_error {
                    Some(
                        errors
                            .iter()
                            .map(|e| FieldError {
                                field: extract_field_from_error(e),
                                message: extract_message_from_error(e),
                            })
                            .collect(),
                    )
                } else {
                    None
                };
                // Use safe_message() to avoid exposing internal details
                (uc_error.status_code(), uc_error.error_code().to_string(), uc_error.safe_message(), details)
            }
            ApiError::BadRequest(msg) => {
                (StatusCode::BAD_REQUEST, "BAD_REQUEST".to_string(), msg.clone(), None)
            }
            ApiError::Unauthorized => {
                (StatusCode::UNAUTHORIZED, "UNAUTHORIZED".to_string(), "Authentication required".to_string(), None)
            }
            ApiError::InvalidUuid(_) => {
                // Don't expose the actual UUID parsing error details
                (StatusCode::BAD_REQUEST, "INVALID_UUID".to_string(), "Invalid ID format".to_string(), None)
            }
            ApiError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR".to_string(),
                "An unexpected error occurred. Please try again later.".to_string(),
                None,
            ),
        };

        let body = ErrorResponse {
            error: ErrorDetail {
                code,
                message,
                details,
            },
            request_id: None, // Will be set by middleware
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        (status, Json(body)).into_response()
    }
}

/// Extract field name from validation error string (format: "field: message")
fn extract_field_from_error(error: &str) -> String {
    error.split(':').next().unwrap_or("").trim().to_string()
}

/// Extract message from validation error string (format: "field: message")
fn extract_message_from_error(error: &str) -> String {
    error.split(':').nth(1).unwrap_or(error).trim().to_string()
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
