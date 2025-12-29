//! Driven Adapters
//!
//! Implementations of gateway traits for external systems:
//! - Database repositories
//! - Configuration
//! - External service clients

pub mod config;
pub mod database;
pub mod network_repository;

pub use config::AppConfig;
pub use network_repository::PostgresNetworkRepository;
