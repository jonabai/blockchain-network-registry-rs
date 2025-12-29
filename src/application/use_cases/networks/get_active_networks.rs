//! Get Active Networks Use Case
//!
//! Retrieves all active networks, sorted by name.

use std::sync::Arc;

use crate::domain::gateways::NetworkRepository;
use crate::domain::models::network::Network;
use crate::shared::errors::UseCaseError;

/// Use case for getting all active networks
pub struct GetActiveNetworksUseCase {
    network_repository: Arc<dyn NetworkRepository>,
}

impl GetActiveNetworksUseCase {
    /// Create a new GetActiveNetworksUseCase
    #[must_use]
    pub fn new(network_repository: Arc<dyn NetworkRepository>) -> Self {
        Self { network_repository }
    }

    /// Execute the use case
    ///
    /// # Errors
    ///
    /// Returns `UseCaseError::Repository` if there's a database error.
    pub async fn execute(&self) -> Result<Vec<Network>, UseCaseError> {
        tracing::debug!("Getting all active networks");

        let networks = self.network_repository.find_all_active().await?;

        tracing::debug!(count = networks.len(), "Found active networks");
        Ok(networks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::network::{CreateNetworkData, NetworkId};
    use crate::shared::errors::RepositoryError;
    use async_trait::async_trait;
    use rust_decimal_macros::dec;
    use std::sync::Mutex;

    struct MockNetworkRepository {
        find_all_active_result: Mutex<Option<Result<Vec<Network>, RepositoryError>>>,
    }

    impl MockNetworkRepository {
        fn new() -> Self {
            Self {
                find_all_active_result: Mutex::new(None),
            }
        }

        fn with_find_all_active(self, result: Result<Vec<Network>, RepositoryError>) -> Self {
            *self.find_all_active_result.lock().unwrap() = Some(result);
            self
        }
    }

    #[async_trait]
    impl NetworkRepository for MockNetworkRepository {
        async fn find_by_id(&self, _id: &NetworkId) -> Result<Option<Network>, RepositoryError> {
            Ok(None)
        }

        async fn find_by_chain_id(&self, _chain_id: i32) -> Result<Option<Network>, RepositoryError> {
            Ok(None)
        }

        async fn find_all_active(&self) -> Result<Vec<Network>, RepositoryError> {
            self.find_all_active_result
                .lock()
                .unwrap()
                .take()
                .unwrap_or(Ok(vec![]))
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

    fn create_test_network(chain_id: i32, name: &str) -> Network {
        Network::new(CreateNetworkData {
            chain_id,
            name: name.to_string(),
            rpc_url: "https://example.com".to_string(),
            other_rpc_urls: vec![],
            test_net: false,
            block_explorer_url: "https://explorer.example.com".to_string(),
            fee_multiplier: dec!(1.0),
            gas_limit_multiplier: dec!(1.2),
            default_signer_address: "0x742d35Cc6634C0532925a3b844Bc9e7595f1dEaD".to_string(),
        })
        .expect("valid test data")
    }

    #[tokio::test]
    async fn should_return_empty_list_when_no_active_networks() {
        let repo = Arc::new(MockNetworkRepository::new().with_find_all_active(Ok(vec![])));

        let use_case = GetActiveNetworksUseCase::new(repo);
        let result = use_case.execute().await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn should_return_active_networks() {
        let networks = vec![
            create_test_network(1, "Ethereum"),
            create_test_network(137, "Polygon"),
        ];
        let repo = Arc::new(MockNetworkRepository::new().with_find_all_active(Ok(networks)));

        let use_case = GetActiveNetworksUseCase::new(repo);
        let result = use_case.execute().await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }
}
