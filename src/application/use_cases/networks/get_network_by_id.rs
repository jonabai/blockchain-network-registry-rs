//! Get Network By ID Use Case
//!
//! Retrieves a single network by its ID.

use std::sync::Arc;

use crate::domain::gateways::NetworkRepository;
use crate::domain::models::network::{Network, NetworkId};
use crate::shared::errors::UseCaseError;

/// Use case for getting a network by ID
pub struct GetNetworkByIdUseCase {
    network_repository: Arc<dyn NetworkRepository>,
}

impl GetNetworkByIdUseCase {
    /// Create a new GetNetworkByIdUseCase
    #[must_use]
    pub fn new(network_repository: Arc<dyn NetworkRepository>) -> Self {
        Self { network_repository }
    }

    /// Execute the use case
    ///
    /// # Errors
    ///
    /// Returns `UseCaseError::NotFound` if the network doesn't exist.
    /// Returns `UseCaseError::Repository` if there's a database error.
    pub async fn execute(&self, id: &NetworkId) -> Result<Network, UseCaseError> {
        tracing::debug!(network_id = %id, "Getting network by ID");

        let network = self.network_repository.find_by_id(id).await?.ok_or_else(|| {
            tracing::warn!(network_id = %id, "Network not found");
            UseCaseError::NotFound {
                resource: "Network".to_string(),
                id: id.to_string(),
            }
        })?;

        tracing::debug!(network_id = %id, "Network found");
        Ok(network)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::network::CreateNetworkData;
    use crate::shared::errors::RepositoryError;
    use async_trait::async_trait;
    use rust_decimal_macros::dec;
    use std::sync::Mutex;

    struct MockNetworkRepository {
        find_by_id_result: Mutex<Option<Result<Option<Network>, RepositoryError>>>,
    }

    impl MockNetworkRepository {
        fn new() -> Self {
            Self {
                find_by_id_result: Mutex::new(None),
            }
        }

        fn with_find_by_id(self, result: Result<Option<Network>, RepositoryError>) -> Self {
            *self.find_by_id_result.lock().unwrap() = Some(result);
            self
        }
    }

    #[async_trait]
    impl NetworkRepository for MockNetworkRepository {
        async fn find_by_id(&self, _id: &NetworkId) -> Result<Option<Network>, RepositoryError> {
            self.find_by_id_result.lock().unwrap().take().unwrap_or(Ok(None))
        }

        async fn find_by_chain_id(&self, _chain_id: i32) -> Result<Option<Network>, RepositoryError> {
            Ok(None)
        }

        async fn find_all_active(&self) -> Result<Vec<Network>, RepositoryError> {
            Ok(vec![])
        }

        async fn create(&self, network: &Network) -> Result<Network, RepositoryError> {
            Ok(network.clone())
        }

        async fn update(&self, _network: &Network) -> Result<Option<Network>, RepositoryError> {
            Ok(None)
        }

        async fn soft_delete(&self, _id: &NetworkId) -> Result<bool, RepositoryError> {
            Ok(false)
        }

        async fn exists_by_chain_id(
            &self,
            _chain_id: i32,
            _exclude_id: Option<&NetworkId>,
        ) -> Result<bool, RepositoryError> {
            Ok(false)
        }
    }

    fn create_test_network() -> Network {
        Network::new(CreateNetworkData {
            chain_id: 1,
            name: "Ethereum Mainnet".to_string(),
            rpc_url: "https://mainnet.infura.io".to_string(),
            other_rpc_urls: vec![],
            test_net: false,
            block_explorer_url: "https://etherscan.io".to_string(),
            fee_multiplier: dec!(1.0),
            gas_limit_multiplier: dec!(1.2),
            default_signer_address: "0x742d35Cc6634C0532925a3b844Bc9e7595f1dEaD".to_string(),
        })
    }

    #[tokio::test]
    async fn should_return_network_when_found() {
        let network = create_test_network();
        let repo = Arc::new(MockNetworkRepository::new().with_find_by_id(Ok(Some(network.clone()))));

        let use_case = GetNetworkByIdUseCase::new(repo);
        let result = use_case.execute(network.id()).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().chain_id(), 1);
    }

    #[tokio::test]
    async fn should_return_not_found_when_network_does_not_exist() {
        let repo = Arc::new(MockNetworkRepository::new().with_find_by_id(Ok(None)));

        let use_case = GetNetworkByIdUseCase::new(repo);
        let result = use_case.execute(&NetworkId::new()).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UseCaseError::NotFound { .. }));
    }
}
