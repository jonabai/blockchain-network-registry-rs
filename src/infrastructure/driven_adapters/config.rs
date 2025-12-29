//! Application Configuration
//!
//! Loads configuration from files and environment variables.
//!
//! Security considerations:
//! - JWT secrets are wrapped in `SecretString` which zeros memory on drop
//! - Sensitive config fields are not cloneable to prevent accidental exposure

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::fmt;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Server configuration
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    /// Allowed CORS origins (empty = localhost only in dev)
    #[serde(default)]
    pub allowed_origins: Vec<String>,
}

/// Database configuration
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

/// A string that zeros its memory when dropped.
/// Does not implement Clone to prevent accidental copying of secrets.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecretString(String);

impl SecretString {
    /// Expose the secret value (use sparingly)
    #[must_use]
    pub fn expose(&self) -> &str {
        &self.0
    }

    /// Get the length of the secret
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if the secret is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

impl<'de> Deserialize<'de> for SecretString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).map(SecretString)
    }
}

/// JWT configuration (not Clone to prevent secret exposure)
#[derive(Debug, Deserialize)]
pub struct JwtConfig {
    pub secret: SecretString,
    pub expires_in_secs: i64,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum requests per second per IP
    pub requests_per_second: u32,
    /// Burst size (max requests allowed in a burst)
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 10,
            burst_size: 50,
        }
    }
}

/// Application configuration (not Clone due to sensitive JWT config)
#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub jwt: JwtConfig,
    #[serde(default)]
    pub rate_limit: RateLimitConfig,
}

/// Minimum required length for JWT secret
const MIN_JWT_SECRET_LENGTH: usize = 32;

impl AppConfig {
    /// Load configuration from files and environment
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Configuration files cannot be loaded
    /// - JWT secret is not provided or too short
    /// - Database URL is not provided
    pub fn load() -> Result<Self, ConfigError> {
        let run_mode = std::env::var("RUN_MODE").unwrap_or_else(|_| "default".into());

        let config: Self = Config::builder()
            // Start with default config
            .add_source(File::with_name("config/default").required(true))
            // Merge environment-specific config if it exists
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            // Override with environment variables (e.g., APP__SERVER__PORT)
            .add_source(Environment::with_prefix("APP").separator("__"))
            .build()?
            .try_deserialize()?;

        // Validate JWT secret (using SecretString methods to avoid exposing the value)
        if config.jwt.secret.is_empty() {
            return Err(ConfigError::Message(
                "JWT secret is required. Set APP__JWT__SECRET environment variable".to_string(),
            ));
        }

        if config.jwt.secret.len() < MIN_JWT_SECRET_LENGTH {
            return Err(ConfigError::Message(format!(
                "JWT secret must be at least {} characters for security. Current length: {}",
                MIN_JWT_SECRET_LENGTH,
                config.jwt.secret.len()
            )));
        }

        // Validate database URL is not empty
        if config.database.url.is_empty() {
            return Err(ConfigError::Message(
                "Database URL is required. Set APP__DATABASE__URL environment variable".to_string(),
            ));
        }

        // Warn if no CORS origins configured (likely development)
        if config.server.allowed_origins.is_empty() {
            eprintln!("WARNING: No CORS allowed_origins configured. Using restrictive defaults.");
        }

        Ok(config)
    }
}
