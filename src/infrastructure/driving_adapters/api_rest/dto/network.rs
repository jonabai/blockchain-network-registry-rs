//! Network DTOs
//!
//! Data transfer objects for network API endpoints.

use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use regex::Regex;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::domain::models::network::{CreateNetworkData, Network, UpdateNetworkData};

lazy_static! {
    /// Regex for validating Ethereum addresses
    static ref ETHEREUM_ADDRESS_REGEX: Regex = Regex::new(r"^0x[a-fA-F0-9]{40}$").expect("valid regex");
}

/// Validates an Ethereum address format
fn validate_ethereum_address(address: &str) -> Result<(), validator::ValidationError> {
    if ETHEREUM_ADDRESS_REGEX.is_match(address) {
        Ok(())
    } else {
        let mut error = validator::ValidationError::new("ethereum_address");
        error.message = Some("Invalid Ethereum address format (must be 0x followed by 40 hex characters)".into());
        Err(error)
    }
}

/// Validates a URL format (must start with http:// or https://)
fn validate_url(url: &str) -> Result<(), validator::ValidationError> {
    // Check protocol
    if !url.starts_with("http://") && !url.starts_with("https://") {
        let mut error = validator::ValidationError::new("url");
        error.message = Some("URL must start with http:// or https://".into());
        return Err(error);
    }

    // Check URL has a host (not just protocol)
    let without_protocol = url.strip_prefix("https://").or_else(|| url.strip_prefix("http://")).unwrap_or("");
    if without_protocol.is_empty() || without_protocol.starts_with('/') {
        let mut error = validator::ValidationError::new("url");
        error.message = Some("URL must include a valid host".into());
        return Err(error);
    }

    Ok(())
}

/// Validates URL list
fn validate_url_list(urls: &[String]) -> Result<(), validator::ValidationError> {
    for url in urls {
        validate_url(url)?;
        if url.len() > 500 {
            let mut error = validator::ValidationError::new("url_length");
            error.message = Some("Each URL must be at most 500 characters".into());
            return Err(error);
        }
    }
    Ok(())
}

/// Validates that an f64 can be safely converted to Decimal
fn validate_decimal(value: f64) -> Result<(), validator::ValidationError> {
    if !value.is_finite() {
        let mut error = validator::ValidationError::new("decimal");
        error.message = Some("Value must be a finite number".into());
        return Err(error);
    }
    if Decimal::try_from(value).is_err() {
        let mut error = validator::ValidationError::new("decimal");
        error.message = Some("Value cannot be represented as a decimal".into());
        return Err(error);
    }
    Ok(())
}

/// Safely converts f64 to Decimal, panics if validation wasn't performed
/// This should only be called after validate() has succeeded
fn f64_to_decimal(value: f64) -> Decimal {
    Decimal::try_from(value).expect("value should have been validated")
}

/// DTO for creating a new network
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateNetworkDto {
    #[validate(range(min = 1, message = "chain_id must be at least 1"))]
    pub chain_id: i32,

    #[validate(length(min = 1, max = 100, message = "name must be between 1 and 100 characters"))]
    pub name: String,

    #[validate(length(max = 500, message = "rpc_url must be at most 500 characters"))]
    #[validate(custom(function = "validate_url"))]
    pub rpc_url: String,

    #[serde(default)]
    #[validate(length(max = 10, message = "other_rpc_urls can have at most 10 items"))]
    #[validate(custom(function = "validate_url_list"))]
    pub other_rpc_urls: Vec<String>,

    pub test_net: bool,

    #[validate(length(max = 500, message = "block_explorer_url must be at most 500 characters"))]
    #[validate(custom(function = "validate_url"))]
    pub block_explorer_url: String,

    #[validate(range(min = 0.0, message = "fee_multiplier must be at least 0"))]
    #[validate(custom(function = "validate_decimal"))]
    pub fee_multiplier: f64,

    #[validate(range(min = 0.0, message = "gas_limit_multiplier must be at least 0"))]
    #[validate(custom(function = "validate_decimal"))]
    pub gas_limit_multiplier: f64,

    #[validate(custom(function = "validate_ethereum_address"))]
    pub default_signer_address: String,
}

impl From<CreateNetworkDto> for CreateNetworkData {
    fn from(dto: CreateNetworkDto) -> Self {
        Self {
            chain_id: dto.chain_id,
            name: dto.name,
            rpc_url: dto.rpc_url,
            other_rpc_urls: dto.other_rpc_urls,
            test_net: dto.test_net,
            block_explorer_url: dto.block_explorer_url,
            fee_multiplier: f64_to_decimal(dto.fee_multiplier),
            gas_limit_multiplier: f64_to_decimal(dto.gas_limit_multiplier),
            default_signer_address: dto.default_signer_address,
        }
    }
}

/// DTO for full network update (PUT)
#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNetworkDto {
    #[validate(range(min = 1, message = "chain_id must be at least 1"))]
    pub chain_id: i32,

    #[validate(length(min = 1, max = 100, message = "name must be between 1 and 100 characters"))]
    pub name: String,

    #[validate(length(max = 500, message = "rpc_url must be at most 500 characters"))]
    #[validate(custom(function = "validate_url"))]
    pub rpc_url: String,

    #[serde(default)]
    #[validate(length(max = 10, message = "other_rpc_urls can have at most 10 items"))]
    #[validate(custom(function = "validate_url_list"))]
    pub other_rpc_urls: Vec<String>,

    pub test_net: bool,

    #[validate(length(max = 500, message = "block_explorer_url must be at most 500 characters"))]
    #[validate(custom(function = "validate_url"))]
    pub block_explorer_url: String,

    #[validate(range(min = 0.0, message = "fee_multiplier must be at least 0"))]
    #[validate(custom(function = "validate_decimal"))]
    pub fee_multiplier: f64,

    #[validate(range(min = 0.0, message = "gas_limit_multiplier must be at least 0"))]
    #[validate(custom(function = "validate_decimal"))]
    pub gas_limit_multiplier: f64,

    #[validate(custom(function = "validate_ethereum_address"))]
    pub default_signer_address: String,
}

impl From<UpdateNetworkDto> for UpdateNetworkData {
    fn from(dto: UpdateNetworkDto) -> Self {
        Self {
            chain_id: Some(dto.chain_id),
            name: Some(dto.name),
            rpc_url: Some(dto.rpc_url),
            other_rpc_urls: Some(dto.other_rpc_urls),
            test_net: Some(dto.test_net),
            block_explorer_url: Some(dto.block_explorer_url),
            fee_multiplier: Some(f64_to_decimal(dto.fee_multiplier)),
            gas_limit_multiplier: Some(f64_to_decimal(dto.gas_limit_multiplier)),
            default_signer_address: Some(dto.default_signer_address),
            active: None, // Cannot update active via PUT
        }
    }
}

/// DTO for partial network update (PATCH)
///
/// All fields are optional. Only provided fields will be updated.
/// Each field is validated if present (validator crate skips None values).
#[derive(Debug, Clone, Deserialize, Validate, Default)]
#[serde(rename_all = "camelCase")]
pub struct PatchNetworkDto {
    #[validate(range(min = 1, message = "chain_id must be at least 1"))]
    pub chain_id: Option<i32>,

    #[validate(length(min = 1, max = 100, message = "name must be between 1 and 100 characters"))]
    pub name: Option<String>,

    #[validate(length(max = 500, message = "rpc_url must be at most 500 characters"))]
    #[validate(custom(function = "validate_url"))]
    pub rpc_url: Option<String>,

    #[validate(length(max = 10, message = "other_rpc_urls can have at most 10 items"))]
    #[validate(custom(function = "validate_url_list"))]
    pub other_rpc_urls: Option<Vec<String>>,

    pub test_net: Option<bool>,

    #[validate(length(max = 500, message = "block_explorer_url must be at most 500 characters"))]
    #[validate(custom(function = "validate_url"))]
    pub block_explorer_url: Option<String>,

    #[validate(range(min = 0.0, message = "fee_multiplier must be at least 0"))]
    #[validate(custom(function = "validate_decimal"))]
    pub fee_multiplier: Option<f64>,

    #[validate(range(min = 0.0, message = "gas_limit_multiplier must be at least 0"))]
    #[validate(custom(function = "validate_decimal"))]
    pub gas_limit_multiplier: Option<f64>,

    #[validate(custom(function = "validate_ethereum_address"))]
    pub default_signer_address: Option<String>,

    pub active: Option<bool>,
}

impl From<PatchNetworkDto> for UpdateNetworkData {
    fn from(dto: PatchNetworkDto) -> Self {
        Self {
            chain_id: dto.chain_id,
            name: dto.name,
            rpc_url: dto.rpc_url,
            other_rpc_urls: dto.other_rpc_urls,
            test_net: dto.test_net,
            block_explorer_url: dto.block_explorer_url,
            fee_multiplier: dto.fee_multiplier.map(f64_to_decimal),
            gas_limit_multiplier: dto.gas_limit_multiplier.map(f64_to_decimal),
            default_signer_address: dto.default_signer_address,
            active: dto.active,
        }
    }
}

/// Network response DTO
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkResponseDto {
    pub id: String,
    pub chain_id: i32,
    pub name: String,
    pub rpc_url: String,
    pub other_rpc_urls: Vec<String>,
    pub test_net: bool,
    pub block_explorer_url: String,
    pub fee_multiplier: f64,
    pub gas_limit_multiplier: f64,
    pub active: bool,
    pub default_signer_address: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Network> for NetworkResponseDto {
    fn from(network: Network) -> Self {
        Self {
            id: network.id().to_string(),
            chain_id: network.chain_id(),
            name: network.name().to_string(),
            rpc_url: network.rpc_url().to_string(),
            other_rpc_urls: network.other_rpc_urls().to_vec(),
            test_net: network.test_net(),
            block_explorer_url: network.block_explorer_url().to_string(),
            fee_multiplier: network.fee_multiplier().try_into().unwrap_or(0.0),
            gas_limit_multiplier: network.gas_limit_multiplier().try_into().unwrap_or(0.0),
            active: network.active(),
            default_signer_address: network.default_signer_address().to_string(),
            created_at: network.created_at(),
            updated_at: network.updated_at(),
        }
    }
}

impl From<&Network> for NetworkResponseDto {
    fn from(network: &Network) -> Self {
        Self {
            id: network.id().to_string(),
            chain_id: network.chain_id(),
            name: network.name().to_string(),
            rpc_url: network.rpc_url().to_string(),
            other_rpc_urls: network.other_rpc_urls().to_vec(),
            test_net: network.test_net(),
            block_explorer_url: network.block_explorer_url().to_string(),
            fee_multiplier: network.fee_multiplier().try_into().unwrap_or(0.0),
            gas_limit_multiplier: network.gas_limit_multiplier().try_into().unwrap_or(0.0),
            active: network.active(),
            default_signer_address: network.default_signer_address().to_string(),
            created_at: network.created_at(),
            updated_at: network.updated_at(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_ethereum_address_valid() {
        assert!(validate_ethereum_address("0x742d35Cc6634C0532925a3b844Bc9e7595f1dEaD").is_ok());
        assert!(validate_ethereum_address("0x0000000000000000000000000000000000000000").is_ok());
    }

    #[test]
    fn test_validate_ethereum_address_invalid() {
        assert!(validate_ethereum_address("invalid").is_err());
        assert!(validate_ethereum_address("0x123").is_err());
        assert!(validate_ethereum_address("742d35Cc6634C0532925a3b844Bc9e7595f1dEaD").is_err());
    }

    #[test]
    fn test_validate_url_valid() {
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("http://localhost:8080").is_ok());
        assert!(validate_url("https://api.example.com/v1").is_ok());
    }

    #[test]
    fn test_validate_url_invalid() {
        assert!(validate_url("ftp://example.com").is_err());
        assert!(validate_url("example.com").is_err());
        // URL with no host
        assert!(validate_url("http://").is_err());
        assert!(validate_url("https://").is_err());
    }

    #[test]
    fn test_validate_decimal_valid() {
        assert!(validate_decimal(1.0).is_ok());
        assert!(validate_decimal(0.0).is_ok());
        assert!(validate_decimal(999999.99).is_ok());
    }

    #[test]
    fn test_validate_decimal_invalid() {
        assert!(validate_decimal(f64::INFINITY).is_err());
        assert!(validate_decimal(f64::NEG_INFINITY).is_err());
        assert!(validate_decimal(f64::NAN).is_err());
    }

    #[test]
    fn test_validate_url_list() {
        assert!(validate_url_list(&[]).is_ok());
        assert!(validate_url_list(&["https://a.com".to_string(), "http://b.com".to_string()]).is_ok());
        assert!(validate_url_list(&["invalid".to_string()]).is_err());
    }

    #[test]
    fn test_patch_dto_validation() {
        // Empty DTO should be valid
        let empty_dto = PatchNetworkDto::default();
        assert!(empty_dto.validate().is_ok());

        // Valid URL should pass
        let dto_with_url = PatchNetworkDto {
            rpc_url: Some("https://example.com".to_string()),
            ..Default::default()
        };
        assert!(dto_with_url.validate().is_ok());

        // Invalid URL should fail
        let dto_with_invalid_url = PatchNetworkDto {
            rpc_url: Some("invalid-url".to_string()),
            ..Default::default()
        };
        assert!(dto_with_invalid_url.validate().is_err());

        // Valid Ethereum address should pass
        let dto_with_address = PatchNetworkDto {
            default_signer_address: Some("0x742d35Cc6634C0532925a3b844Bc9e7595f1dEaD".to_string()),
            ..Default::default()
        };
        assert!(dto_with_address.validate().is_ok());

        // Invalid Ethereum address should fail
        let dto_with_invalid_address = PatchNetworkDto {
            default_signer_address: Some("invalid".to_string()),
            ..Default::default()
        };
        assert!(dto_with_invalid_address.validate().is_err());

        // Valid decimal should pass
        let dto_with_decimal = PatchNetworkDto {
            fee_multiplier: Some(1.5),
            ..Default::default()
        };
        assert!(dto_with_decimal.validate().is_ok());

        // Infinity should fail
        let dto_with_infinity = PatchNetworkDto {
            fee_multiplier: Some(f64::INFINITY),
            ..Default::default()
        };
        assert!(dto_with_infinity.validate().is_err());
    }
}
