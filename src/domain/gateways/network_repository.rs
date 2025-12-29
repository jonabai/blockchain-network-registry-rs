//! Network Repository Gateway
//!
//! Abstract trait defining the contract for network persistence operations.

use async_trait::async_trait;

use crate::domain::models::network::{Network, NetworkId};
use crate::shared::errors::RepositoryError;

/// Repository trait for Network persistence operations
#[async_trait]
pub trait NetworkRepository: Send + Sync {
    /// Find a network by its ID
    async fn find_by_id(&self, id: &NetworkId) -> Result<Option<Network>, RepositoryError>;

    /// Find a network by its chain ID
    async fn find_by_chain_id(&self, chain_id: i32) -> Result<Option<Network>, RepositoryError>;

    /// Find all active networks, sorted by name ascending
    async fn find_all_active(&self) -> Result<Vec<Network>, RepositoryError>;

    /// Create a new network
    async fn create(&self, network: &Network) -> Result<Network, RepositoryError>;

    /// Update an existing network
    async fn update(&self, network: &Network) -> Result<Option<Network>, RepositoryError>;

    /// Soft delete a network (sets active=false)
    async fn soft_delete(&self, id: &NetworkId) -> Result<bool, RepositoryError>;

    /// Check if a chain ID exists, optionally excluding a specific network ID
    async fn exists_by_chain_id(
        &self,
        chain_id: i32,
        exclude_id: Option<&NetworkId>,
    ) -> Result<bool, RepositoryError>;
}
