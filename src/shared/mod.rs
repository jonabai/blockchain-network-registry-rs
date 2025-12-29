//! Shared Module
//!
//! Cross-cutting utilities and types used across the application.

pub mod errors;

pub use errors::{ApiError, DomainError, RepositoryError, UseCaseError};
