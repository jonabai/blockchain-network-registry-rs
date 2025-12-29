//! JWT Authentication Middleware
//!
//! Extracts and validates JWT tokens from requests.

use std::sync::Arc;

use axum::{
    extract::{FromRequestParts, State},
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

use crate::infrastructure::driven_adapters::config::AppConfig;
use crate::shared::errors::ErrorResponse;

/// JWT claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Email
    pub email: String,
    /// Role
    pub role: String,
    /// Issued at timestamp
    pub iat: i64,
    /// Expiration timestamp
    pub exp: i64,
}

/// Authenticated user extracted from JWT
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: String,
    pub email: String,
    pub role: String,
}

impl From<Claims> for AuthenticatedUser {
    fn from(claims: Claims) -> Self {
        Self {
            id: claims.sub,
            email: claims.email,
            role: claims.role,
        }
    }
}

/// JWT authentication extractor
pub struct JwtAuth(pub AuthenticatedUser);

/// Error type for authentication failures
pub struct AuthError {
    message: String,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let body = ErrorResponse {
            error: crate::shared::errors::ErrorDetail {
                code: "UNAUTHORIZED".to_string(),
                message: self.message,
                details: None,
            },
            request_id: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        (StatusCode::UNAUTHORIZED, Json(body)).into_response()
    }
}

impl<S> FromRequestParts<S> for JwtAuth
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    fn from_request_parts<'life0, 'life1, 'async_trait>(
        parts: &'life0 mut Parts,
        _state: &'life1 S,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self, Self::Rejection>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            // Get config from request extensions
            let config = parts
                .extensions
                .get::<Arc<AppConfig>>()
                .ok_or_else(|| AuthError {
                    message: "Configuration not available".to_string(),
                })?
                .clone();

            // Extract Authorization header
            let auth_header = parts
                .headers
                .get(AUTHORIZATION)
                .and_then(|h| h.to_str().ok())
                .ok_or_else(|| AuthError {
                    message: "Missing Authorization header".to_string(),
                })?;

            // Check Bearer prefix
            let token = auth_header
                .strip_prefix("Bearer ")
                .ok_or_else(|| AuthError {
                    message: "Invalid Authorization header format".to_string(),
                })?;

            // Decode and validate JWT
            let token_data = decode::<Claims>(
                token,
                &DecodingKey::from_secret(config.jwt.secret.as_bytes()),
                &Validation::default(),
            )
            .map_err(|e| AuthError {
                message: format!("Invalid token: {}", e),
            })?;

            Ok(JwtAuth(token_data.claims.into()))
        })
    }
}

/// Middleware layer that adds config to request extensions
pub async fn add_config_extension(
    State(config): State<Arc<AppConfig>>,
    mut request: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> Response {
    request.extensions_mut().insert(config);
    next.run(request).await
}
