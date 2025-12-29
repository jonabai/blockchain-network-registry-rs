//! Gateway Traits (Ports)
//!
//! Abstract interfaces defining contracts for external dependencies.
//! These are implemented by driven adapters in the infrastructure layer.

pub mod network_repository;

pub use network_repository::NetworkRepository;
