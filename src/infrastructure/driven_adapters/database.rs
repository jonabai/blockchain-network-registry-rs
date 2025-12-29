//! Database Connection Management
//!
//! Utilities for creating and managing database connections.

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use super::config::DatabaseConfig;

/// Create a PostgreSQL connection pool from configuration
pub async fn create_pool(config: &DatabaseConfig) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .connect(&config.url)
        .await
}
