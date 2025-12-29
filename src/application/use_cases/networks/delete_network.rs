//! Delete Network Use Case (Soft Delete)
//!
//! Soft deletes a network by setting active=false.

use std::sync::Arc;

use crate::domain::gateways::NetworkRepository;
use crate::domain::models::network::NetworkId;
use crate::shared::errors::UseCaseError;

/// Use case for soft deleting a network
pub struct DeleteNetworkUseCase {
    network_repository: Arc<dyn NetworkRepository>,
}

impl DeleteNetworkUseCase {
    /// Create a new DeleteNetworkUseCase
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
    pub async fn execute(&self, id: &NetworkId) -> Result<(), UseCaseError> {
        tracing::info!(network_id = %id, "Soft deleting network");

        let deleted = self.network_repository.soft_delete(id).await?;

        if !deleted {
            tracing::warn!(network_id = %id, "Network not found for deletion");
            return Err(UseCaseError::NotFound {
                resource: "Network".to_string(),
                id: id.to_string(),
            });
        }

        tracing::info!(network_id = %id, "Network soft deleted successfully");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::network::Network;
    use crate::shared::errors::RepositoryError;
    use async_trait::async_trait;
    use std::sync::Mutex;

    struct MockNetworkRepository {
        soft_delete_result: Mutex<Option<Result<bool, RepositoryError>>>,
    }

    impl MockNetworkRepository {
        fn new() -> Self {
            Self {
                soft_delete_result: Mutex::new(None),
            }
        }

        fn with_soft_delete(self, result: Result<bool, RepositoryError>) -> Self {
            *self.soft_delete_result.lock().unwrap() = Some(result);
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
            Ok(network.clone())
        }

        async fn update(&self, _network: &Network) -> Result<Option<Network>, RepositoryError> {
            Ok(None)
        }

        async fn soft_delete(&self, _id: &NetworkId) -> Result<bool, RepositoryError> {
            self.soft_delete_result
                .lock()
                .unwrap()
                .take()
                .unwrap_or(Ok(false))
        }

        async fn exists_by_chain_id(
            &self,
            _chain_id: i32,
            _exclude_id: Option<&NetworkId>,
        ) -> Result<bool, RepositoryError> {
            Ok(false)
        }
    }

    #[tokio::test]
    async fn should_soft_delete_network_when_found() {
        let repo = Arc::new(MockNetworkRepository::new().with_soft_delete(Ok(true)));

        let use_case = DeleteNetworkUseCase::new(repo);
        let result = use_case.execute(&NetworkId::new()).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_return_not_found_when_network_does_not_exist() {
        let repo = Arc::new(MockNetworkRepository::new().with_soft_delete(Ok(false)));

        let use_case = DeleteNetworkUseCase::new(repo);
        let result = use_case.execute(&NetworkId::new()).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UseCaseError::NotFound { .. }));
    }
}
