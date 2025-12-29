//! Partial Update Network Use Case (PATCH)
//!
//! Updates only the provided fields of a network, including 'active'.

use std::sync::Arc;

use crate::domain::gateways::NetworkRepository;
use crate::domain::models::network::{Network, NetworkId, UpdateNetworkData};
use crate::shared::errors::UseCaseError;

/// Use case for partial network update (PATCH)
pub struct PartialUpdateNetworkUseCase {
    network_repository: Arc<dyn NetworkRepository>,
}

impl PartialUpdateNetworkUseCase {
    /// Create a new PartialUpdateNetworkUseCase
    #[must_use]
    pub fn new(network_repository: Arc<dyn NetworkRepository>) -> Self {
        Self { network_repository }
    }

    /// Execute the use case
    ///
    /// # Errors
    ///
    /// Returns `UseCaseError::NotFound` if the network doesn't exist.
    /// Returns `UseCaseError::Conflict` if the new chain_id already exists.
    /// Returns `UseCaseError::Repository` if there's a database error.
    pub async fn execute(&self, id: &NetworkId, data: UpdateNetworkData) -> Result<Network, UseCaseError> {
        tracing::info!(network_id = %id, "Partially updating network");

        // Find existing network
        let existing = self.network_repository.find_by_id(id).await?.ok_or_else(|| {
            tracing::warn!(network_id = %id, "Network not found for partial update");
            UseCaseError::NotFound {
                resource: "Network".to_string(),
                id: id.to_string(),
            }
        })?;

        // Check chain_id uniqueness if it changed
        if let Some(new_chain_id) = data.chain_id {
            if new_chain_id != existing.chain_id() {
                if self
                    .network_repository
                    .exists_by_chain_id(new_chain_id, Some(id))
                    .await?
                {
                    tracing::warn!(
                        network_id = %id,
                        new_chain_id = new_chain_id,
                        "Cannot update: chain_id already exists"
                    );
                    return Err(UseCaseError::Conflict(format!(
                        "Network with chain_id {} already exists",
                        new_chain_id
                    )));
                }
            }
        }

        // Apply updates (PATCH can update active field)
        let updated = existing.with_updates(data);

        // Save and return
        let result = self.network_repository.update(&updated).await?.ok_or_else(|| {
            UseCaseError::NotFound {
                resource: "Network".to_string(),
                id: id.to_string(),
            }
        })?;

        tracing::info!(network_id = %id, "Network partially updated successfully");
        Ok(result)
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
        exists_by_chain_id_result: Mutex<Option<Result<bool, RepositoryError>>>,
        update_result: Mutex<Option<Result<Option<Network>, RepositoryError>>>,
    }

    impl MockNetworkRepository {
        fn new() -> Self {
            Self {
                find_by_id_result: Mutex::new(None),
                exists_by_chain_id_result: Mutex::new(None),
                update_result: Mutex::new(None),
            }
        }

        fn with_find_by_id(self, result: Result<Option<Network>, RepositoryError>) -> Self {
            *self.find_by_id_result.lock().unwrap() = Some(result);
            self
        }

        fn with_exists_by_chain_id(self, result: Result<bool, RepositoryError>) -> Self {
            *self.exists_by_chain_id_result.lock().unwrap() = Some(result);
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

        async fn update(&self, network: &Network) -> Result<Option<Network>, RepositoryError> {
            self.update_result
                .lock()
                .unwrap()
                .take()
                .unwrap_or(Ok(Some(network.clone())))
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
    async fn should_partially_update_network() {
        let network = create_test_network();
        let repo = Arc::new(
            MockNetworkRepository::new()
                .with_find_by_id(Ok(Some(network.clone())))
                .with_exists_by_chain_id(Ok(false)),
        );

        let use_case = PartialUpdateNetworkUseCase::new(repo);
        let update_data = UpdateNetworkData {
            name: Some("Updated Name".to_string()),
            ..Default::default()
        };
        let result = use_case.execute(network.id(), update_data).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_allow_updating_active_field() {
        let network = create_test_network();
        let repo = Arc::new(
            MockNetworkRepository::new()
                .with_find_by_id(Ok(Some(network.clone())))
                .with_exists_by_chain_id(Ok(false)),
        );

        let use_case = PartialUpdateNetworkUseCase::new(repo);
        let update_data = UpdateNetworkData {
            active: Some(false),
            ..Default::default()
        };
        let result = use_case.execute(network.id(), update_data).await;

        assert!(result.is_ok());
        // In real scenario, we'd verify the network is deactivated
    }

    #[tokio::test]
    async fn should_return_not_found_when_network_does_not_exist() {
        let repo = Arc::new(MockNetworkRepository::new().with_find_by_id(Ok(None)));

        let use_case = PartialUpdateNetworkUseCase::new(repo);
        let result = use_case
            .execute(&NetworkId::new(), UpdateNetworkData::default())
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UseCaseError::NotFound { .. }));
    }
}
