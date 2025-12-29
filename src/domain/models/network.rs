//! Network Domain Model
//!
//! Represents a blockchain network in the registry.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::shared::errors::DomainError;

/// Maximum length for network name
pub const MAX_NAME_LENGTH: usize = 100;
/// Maximum length for RPC URL
pub const MAX_URL_LENGTH: usize = 500;
/// Maximum number of other RPC URLs
pub const MAX_OTHER_RPC_URLS: usize = 10;
/// Ethereum address length (0x + 40 hex chars)
pub const ETHEREUM_ADDRESS_LENGTH: usize = 42;

/// Newtype wrapper for Network ID providing type safety
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NetworkId(Uuid);

impl NetworkId {
    /// Create a new random NetworkId
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create a NetworkId from an existing UUID
    #[must_use]
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the underlying UUID
    #[must_use]
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    /// Convert to string representation
    #[must_use]
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl Default for NetworkId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for NetworkId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for NetworkId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl TryFrom<&str> for NetworkId {
    type Error = uuid::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Self(Uuid::parse_str(value)?))
    }
}

impl TryFrom<String> for NetworkId {
    type Error = uuid::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

/// Data required to create a new Network
#[derive(Debug, Clone)]
pub struct CreateNetworkData {
    pub chain_id: i32,
    pub name: String,
    pub rpc_url: String,
    pub other_rpc_urls: Vec<String>,
    pub test_net: bool,
    pub block_explorer_url: String,
    pub fee_multiplier: Decimal,
    pub gas_limit_multiplier: Decimal,
    pub default_signer_address: String,
}

impl CreateNetworkData {
    /// Validate all fields in the creation data
    ///
    /// # Errors
    ///
    /// Returns a `DomainError::ValidationError` if any field is invalid
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.chain_id < 1 {
            return Err(DomainError::ValidationError("chain_id must be at least 1".to_string()));
        }

        if self.name.is_empty() || self.name.len() > MAX_NAME_LENGTH {
            return Err(DomainError::ValidationError(format!(
                "name must be between 1 and {} characters",
                MAX_NAME_LENGTH
            )));
        }

        if self.rpc_url.len() > MAX_URL_LENGTH {
            return Err(DomainError::ValidationError(format!(
                "rpc_url must be at most {} characters",
                MAX_URL_LENGTH
            )));
        }

        if self.other_rpc_urls.len() > MAX_OTHER_RPC_URLS {
            return Err(DomainError::ValidationError(format!(
                "other_rpc_urls can have at most {} items",
                MAX_OTHER_RPC_URLS
            )));
        }

        for url in &self.other_rpc_urls {
            if url.len() > MAX_URL_LENGTH {
                return Err(DomainError::ValidationError(format!(
                    "each URL in other_rpc_urls must be at most {} characters",
                    MAX_URL_LENGTH
                )));
            }
        }

        if self.block_explorer_url.len() > MAX_URL_LENGTH {
            return Err(DomainError::ValidationError(format!(
                "block_explorer_url must be at most {} characters",
                MAX_URL_LENGTH
            )));
        }

        if self.fee_multiplier < Decimal::ZERO {
            return Err(DomainError::ValidationError("fee_multiplier must be at least 0".to_string()));
        }

        if self.gas_limit_multiplier < Decimal::ZERO {
            return Err(DomainError::ValidationError("gas_limit_multiplier must be at least 0".to_string()));
        }

        if self.default_signer_address.len() != ETHEREUM_ADDRESS_LENGTH {
            return Err(DomainError::ValidationError(format!(
                "default_signer_address must be {} characters",
                ETHEREUM_ADDRESS_LENGTH
            )));
        }

        Ok(())
    }
}

/// Data for updating an existing Network (all fields optional for partial updates)
#[derive(Debug, Clone, Default)]
pub struct UpdateNetworkData {
    pub chain_id: Option<i32>,
    pub name: Option<String>,
    pub rpc_url: Option<String>,
    pub other_rpc_urls: Option<Vec<String>>,
    pub test_net: Option<bool>,
    pub block_explorer_url: Option<String>,
    pub fee_multiplier: Option<Decimal>,
    pub gas_limit_multiplier: Option<Decimal>,
    pub default_signer_address: Option<String>,
    pub active: Option<bool>,
}

/// Network domain entity representing a blockchain network
#[derive(Debug, Clone)]
pub struct Network {
    id: NetworkId,
    chain_id: i32,
    name: String,
    rpc_url: String,
    other_rpc_urls: Vec<String>,
    test_net: bool,
    block_explorer_url: String,
    fee_multiplier: Decimal,
    gas_limit_multiplier: Decimal,
    active: bool,
    default_signer_address: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Network {
    /// Create a new Network from creation data
    ///
    /// # Errors
    ///
    /// Returns a `DomainError::ValidationError` if the data is invalid
    pub fn new(data: CreateNetworkData) -> Result<Self, DomainError> {
        // Validate data at the domain level
        data.validate()?;

        let now = Utc::now();
        Ok(Self {
            id: NetworkId::new(),
            chain_id: data.chain_id,
            name: data.name,
            rpc_url: data.rpc_url,
            other_rpc_urls: data.other_rpc_urls,
            test_net: data.test_net,
            block_explorer_url: data.block_explorer_url,
            fee_multiplier: data.fee_multiplier,
            gas_limit_multiplier: data.gas_limit_multiplier,
            active: true,
            default_signer_address: data.default_signer_address,
            created_at: now,
            updated_at: now,
        })
    }

    /// Restore a Network from persisted data
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn restore(
        id: NetworkId,
        chain_id: i32,
        name: String,
        rpc_url: String,
        other_rpc_urls: Vec<String>,
        test_net: bool,
        block_explorer_url: String,
        fee_multiplier: Decimal,
        gas_limit_multiplier: Decimal,
        active: bool,
        default_signer_address: String,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            chain_id,
            name,
            rpc_url,
            other_rpc_urls,
            test_net,
            block_explorer_url,
            fee_multiplier,
            gas_limit_multiplier,
            active,
            default_signer_address,
            created_at,
            updated_at,
        }
    }

    /// Apply updates to the network, returning a new instance
    #[must_use]
    pub fn with_updates(self, data: UpdateNetworkData) -> Self {
        Self {
            id: self.id,
            chain_id: data.chain_id.unwrap_or(self.chain_id),
            name: data.name.unwrap_or(self.name),
            rpc_url: data.rpc_url.unwrap_or(self.rpc_url),
            other_rpc_urls: data.other_rpc_urls.unwrap_or(self.other_rpc_urls),
            test_net: data.test_net.unwrap_or(self.test_net),
            block_explorer_url: data.block_explorer_url.unwrap_or(self.block_explorer_url),
            fee_multiplier: data.fee_multiplier.unwrap_or(self.fee_multiplier),
            gas_limit_multiplier: data.gas_limit_multiplier.unwrap_or(self.gas_limit_multiplier),
            active: data.active.unwrap_or(self.active),
            default_signer_address: data.default_signer_address.unwrap_or(self.default_signer_address),
            created_at: self.created_at,
            updated_at: Utc::now(),
        }
    }

    /// Mark the network as inactive (soft delete)
    #[must_use]
    pub fn deactivate(self) -> Self {
        Self {
            active: false,
            updated_at: Utc::now(),
            ..self
        }
    }

    // Getters

    #[must_use]
    pub fn id(&self) -> &NetworkId {
        &self.id
    }

    #[must_use]
    pub fn chain_id(&self) -> i32 {
        self.chain_id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn rpc_url(&self) -> &str {
        &self.rpc_url
    }

    #[must_use]
    pub fn other_rpc_urls(&self) -> &[String] {
        &self.other_rpc_urls
    }

    #[must_use]
    pub fn test_net(&self) -> bool {
        self.test_net
    }

    #[must_use]
    pub fn block_explorer_url(&self) -> &str {
        &self.block_explorer_url
    }

    #[must_use]
    pub fn fee_multiplier(&self) -> Decimal {
        self.fee_multiplier
    }

    #[must_use]
    pub fn gas_limit_multiplier(&self) -> Decimal {
        self.gas_limit_multiplier
    }

    #[must_use]
    pub fn active(&self) -> bool {
        self.active
    }

    #[must_use]
    pub fn default_signer_address(&self) -> &str {
        &self.default_signer_address
    }

    #[must_use]
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    #[must_use]
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn create_test_network_data() -> CreateNetworkData {
        CreateNetworkData {
            chain_id: 1,
            name: "Ethereum Mainnet".to_string(),
            rpc_url: "https://mainnet.infura.io/v3/YOUR-PROJECT-ID".to_string(),
            other_rpc_urls: vec!["https://eth.llamarpc.com".to_string()],
            test_net: false,
            block_explorer_url: "https://etherscan.io".to_string(),
            fee_multiplier: dec!(1.0),
            gas_limit_multiplier: dec!(1.2),
            default_signer_address: "0x742d35Cc6634C0532925a3b844Bc9e7595f1dEaD".to_string(),
        }
    }

    #[test]
    fn test_network_id_new() {
        let id1 = NetworkId::new();
        let id2 = NetworkId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_network_id_from_uuid() {
        let uuid = Uuid::new_v4();
        let id = NetworkId::from_uuid(uuid);
        assert_eq!(id.as_uuid(), &uuid);
    }

    #[test]
    fn test_network_id_try_from_string() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let id = NetworkId::try_from(uuid_str).unwrap();
        assert_eq!(id.to_string(), uuid_str);
    }

    #[test]
    fn test_network_new() {
        let data = create_test_network_data();
        let network = Network::new(data.clone()).expect("valid data should create network");

        assert_eq!(network.chain_id(), data.chain_id);
        assert_eq!(network.name(), data.name);
        assert_eq!(network.rpc_url(), data.rpc_url);
        assert_eq!(network.other_rpc_urls(), data.other_rpc_urls.as_slice());
        assert_eq!(network.test_net(), data.test_net);
        assert_eq!(network.block_explorer_url(), data.block_explorer_url);
        assert_eq!(network.fee_multiplier(), data.fee_multiplier);
        assert_eq!(network.gas_limit_multiplier(), data.gas_limit_multiplier);
        assert!(network.active());
        assert_eq!(network.default_signer_address(), data.default_signer_address);
    }

    #[test]
    fn test_network_new_validates_chain_id() {
        let mut data = create_test_network_data();
        data.chain_id = 0;
        let result = Network::new(data);
        assert!(result.is_err());
    }

    #[test]
    fn test_network_new_validates_name_length() {
        let mut data = create_test_network_data();
        data.name = "x".repeat(MAX_NAME_LENGTH + 1);
        let result = Network::new(data);
        assert!(result.is_err());
    }

    #[test]
    fn test_network_new_validates_empty_name() {
        let mut data = create_test_network_data();
        data.name = String::new();
        let result = Network::new(data);
        assert!(result.is_err());
    }

    #[test]
    fn test_network_new_validates_signer_address_length() {
        let mut data = create_test_network_data();
        data.default_signer_address = "0x123".to_string(); // too short
        let result = Network::new(data);
        assert!(result.is_err());
    }

    #[test]
    fn test_network_with_updates() {
        let data = create_test_network_data();
        let network = Network::new(data).expect("valid data");

        let updates = UpdateNetworkData {
            name: Some("Updated Network".to_string()),
            chain_id: Some(2),
            ..Default::default()
        };

        let updated_network = network.with_updates(updates);
        assert_eq!(updated_network.name(), "Updated Network");
        assert_eq!(updated_network.chain_id(), 2);
    }

    #[test]
    fn test_network_deactivate() {
        let data = create_test_network_data();
        let network = Network::new(data).expect("valid data");
        assert!(network.active());

        let deactivated = network.deactivate();
        assert!(!deactivated.active());
    }
}
