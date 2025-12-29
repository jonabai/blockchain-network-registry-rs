//! Domain Layer
//!
//! Contains the core business logic, domain models, and gateway traits (ports).
//! This layer has no dependencies on infrastructure.

pub mod gateways;
pub mod models;

pub use gateways::network_repository::NetworkRepository;
pub use models::network::{CreateNetworkData, Network, NetworkId, UpdateNetworkData};
