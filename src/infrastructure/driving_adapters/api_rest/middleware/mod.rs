//! API Middleware
//!
//! Authentication and other middleware for the REST API.

pub mod auth;

pub use auth::{AuthenticatedUser, JwtAuth};
