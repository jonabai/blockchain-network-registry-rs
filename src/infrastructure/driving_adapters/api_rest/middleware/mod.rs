//! API Middleware
//!
//! Authentication and other middleware for the REST API.

pub mod auth;
pub mod request_id;

pub use auth::{AuthenticatedUser, JwtAuth};
pub use request_id::{request_id_middleware, RequestId, REQUEST_ID_HEADER};
