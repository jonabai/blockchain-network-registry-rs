//! Create Network Use Case
//!
//! Creates a new blockchain network in the registry.

use std::sync::Arc;

use crate::domain::gateways::NetworkRepository;
use crate::domain::models::network::{CreateNetworkData, Network};
use crate::shared::errors::UseCaseError;

/// Use case for creating a new network
pub struct CreateNetworkUseCase {
    network_repository: Arc<dyn NetworkRepository>,
}

impl CreateNetworkUseCase {
    /// Create a new CreateNetworkUseCase
    #[must_use]
    pub fn new(network_repository: Arc<dyn NetworkRepository>) -> Self {
        Self { network_repository }
    }

    /// Execute the use case
    ///
    /// # Errors
    ///
    /// Returns `UseCaseError::Conflict` if a network with the same chain_id already exists.
    /// Returns `UseCaseError::Repository` if there's a database error.
    pub async fn execute(&self, data: CreateNetworkData) -> Result<Network, UseCaseError> {
        tracing::info!(chain_id = data.chain_id, name = %data.name, "Creating new network");

        // Check if chain_id already exists
        if self
            .network_repository
            .exists_by_chain_id(data.chain_id, None)
            .await?
        {
            tracing::warn!(chain_id = data.chain_id, "Network with chain_id already exists");
            return Err(UseCaseError::Conflict(format!(
                "Network with chain_id {} already exists",
                data.chain_id
            )));
        }

        // Create the network (validates domain constraints)
        let network = Network::new(data)?;
        let created = self.network_repository.create(&network).await?;

        tracing::info!(
            network_id = %created.id(),
            chain_id = created.chain_id(),
            "Network created successfully"
        );

        Ok(created)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::network::NetworkId;
    use crate::shared::errors::RepositoryError;
    use async_trait::async_trait;
    use rust_decimal_macros::dec;
    use std::sync::Mutex;

    struct MockNetworkRepository {
        exists_by_chain_id_result: Mutex<Option<Result<bool, RepositoryError>>>,
        create_result: Mutex<Option<Result<Network, RepositoryError>>>,
    }

    impl MockNetworkRepository {
        fn new() -> Self {
            Self {
                exists_by_chain_id_result: Mutex::new(None),
                create_result: Mutex::new(None),
            }
        }

        fn with_exists_by_chain_id(self, result: Result<bool, RepositoryError>) -> Self {
            *self.exists_by_chain_id_result.lock().unwrap() = Some(result);
            self
        }

        fn with_create(self, result: Result<Network, RepositoryError>) -> Self {
            *self.create_result.lock().unwrap() = Some(result);
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
            Ok(vec![])
        }

        async fn create(&self, network: &Network) -> Result<Network, RepositoryError> {
            self.create_result
                .lock()
                .unwrap()
                .take()
                .unwrap_or_else(|| Ok(network.clone()))
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
            self.exists_by_chain_id_result
                .lock()
                .unwrap()
                .take()
                .unwrap_or(Ok(false))
        }
    }

    fn create_test_data() -> CreateNetworkData {
        CreateNetworkData {
            chain_id: 1,
            name: "Ethereum Mainnet".to_string(),
            rpc_url: "https://mainnet.infura.io".to_string(),
            other_rpc_urls: vec![],
            test_net: false,
            block_explorer_url: "https://etherscan.io".to_string(),
            fee_multiplier: dec!(1.0),
            gas_limit_multiplier: dec!(1.2),
            default_signer_address: "0x742d35Cc6634C0532925a3b844Bc9e7595f1dEaD".to_string(),
        }
    }

    #[tokio::test]
    async fn should_create_network_when_chain_id_does_not_exist() {
        let repo = Arc::new(MockNetworkRepository::new().with_exists_by_chain_id(Ok(false)));

        let use_case = CreateNetworkUseCase::new(repo);
        let result = use_case.execute(create_test_data()).await;

        assert!(result.is_ok());
        let network = result.unwrap();
        assert_eq!(network.chain_id(), 1);
        assert_eq!(network.name(), "Ethereum Mainnet");
    }

    #[tokio::test]
    async fn should_return_conflict_when_chain_id_exists() {
        let repo = Arc::new(MockNetworkRepository::new().with_exists_by_chain_id(Ok(true)));

        let use_case = CreateNetworkUseCase::new(repo);
        let result = use_case.execute(create_test_data()).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UseCaseError::Conflict(_)));
    }
}
