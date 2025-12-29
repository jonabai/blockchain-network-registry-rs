//! PostgreSQL Network Repository Implementation
//!
//! Implements the NetworkRepository trait using SQLx for PostgreSQL.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::gateways::NetworkRepository;
use crate::domain::models::network::{Network, NetworkId};
use crate::shared::errors::RepositoryError;

/// Database row representation for network table
#[derive(Debug, sqlx::FromRow)]
struct NetworkRow {
    id: Uuid,
    chain_id: i32,
    name: String,
    rpc_url: String,
    other_rpc_urls: serde_json::Value,
    test_net: bool,
    block_explorer_url: String,
    fee_multiplier: Decimal,
    gas_limit_multiplier: Decimal,
    active: bool,
    default_signer_address: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<NetworkRow> for Network {
    type Error = RepositoryError;

    fn try_from(row: NetworkRow) -> Result<Self, Self::Error> {
        let other_rpc_urls: Vec<String> = serde_json::from_value(row.other_rpc_urls)
            .map_err(|e| RepositoryError::Mapping(format!("Failed to parse other_rpc_urls: {}", e)))?;

        Ok(Network::restore(
            NetworkId::from_uuid(row.id),
            row.chain_id,
            row.name,
            row.rpc_url,
            other_rpc_urls,
            row.test_net,
            row.block_explorer_url,
            row.fee_multiplier,
            row.gas_limit_multiplier,
            row.active,
            row.default_signer_address,
            row.created_at,
            row.updated_at,
        ))
    }
}

/// PostgreSQL implementation of NetworkRepository
pub struct PostgresNetworkRepository {
    pool: PgPool,
}

impl PostgresNetworkRepository {
    /// Create a new PostgresNetworkRepository
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl NetworkRepository for PostgresNetworkRepository {
    async fn find_by_id(&self, id: &NetworkId) -> Result<Option<Network>, RepositoryError> {
        let row = sqlx::query_as::<_, NetworkRow>(
            r#"
            SELECT id, chain_id, name, rpc_url, other_rpc_urls, test_net,
                   block_explorer_url, fee_multiplier, gas_limit_multiplier,
                   active, default_signer_address, created_at, updated_at
            FROM networks
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await?;

        row.map(Network::try_from).transpose()
    }

    async fn find_by_chain_id(&self, chain_id: i32) -> Result<Option<Network>, RepositoryError> {
        let row = sqlx::query_as::<_, NetworkRow>(
            r#"
            SELECT id, chain_id, name, rpc_url, other_rpc_urls, test_net,
                   block_explorer_url, fee_multiplier, gas_limit_multiplier,
                   active, default_signer_address, created_at, updated_at
            FROM networks
            WHERE chain_id = $1
            "#,
        )
        .bind(chain_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(Network::try_from).transpose()
    }

    async fn find_all_active(&self) -> Result<Vec<Network>, RepositoryError> {
        let rows = sqlx::query_as::<_, NetworkRow>(
            r#"
            SELECT id, chain_id, name, rpc_url, other_rpc_urls, test_net,
                   block_explorer_url, fee_multiplier, gas_limit_multiplier,
                   active, default_signer_address, created_at, updated_at
            FROM networks
            WHERE active = true
            ORDER BY name ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(Network::try_from).collect()
    }

    async fn create(&self, network: &Network) -> Result<Network, RepositoryError> {
        let other_rpc_urls_json = serde_json::to_value(network.other_rpc_urls())
            .map_err(|e| RepositoryError::Mapping(format!("Failed to serialize other_rpc_urls: {}", e)))?;

        let row = sqlx::query_as::<_, NetworkRow>(
            r#"
            INSERT INTO networks (
                id, chain_id, name, rpc_url, other_rpc_urls, test_net,
                block_explorer_url, fee_multiplier, gas_limit_multiplier,
                active, default_signer_address, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING id, chain_id, name, rpc_url, other_rpc_urls, test_net,
                      block_explorer_url, fee_multiplier, gas_limit_multiplier,
                      active, default_signer_address, created_at, updated_at
            "#,
        )
        .bind(network.id().as_uuid())
        .bind(network.chain_id())
        .bind(network.name())
        .bind(network.rpc_url())
        .bind(&other_rpc_urls_json)
        .bind(network.test_net())
        .bind(network.block_explorer_url())
        .bind(network.fee_multiplier())
        .bind(network.gas_limit_multiplier())
        .bind(network.active())
        .bind(network.default_signer_address())
        .bind(network.created_at())
        .bind(network.updated_at())
        .fetch_one(&self.pool)
        .await?;

        Network::try_from(row)
    }

    async fn update(&self, network: &Network) -> Result<Option<Network>, RepositoryError> {
        let other_rpc_urls_json = serde_json::to_value(network.other_rpc_urls())
            .map_err(|e| RepositoryError::Mapping(format!("Failed to serialize other_rpc_urls: {}", e)))?;

        let row = sqlx::query_as::<_, NetworkRow>(
            r#"
            UPDATE networks
            SET chain_id = $2,
                name = $3,
                rpc_url = $4,
                other_rpc_urls = $5,
                test_net = $6,
                block_explorer_url = $7,
                fee_multiplier = $8,
                gas_limit_multiplier = $9,
                active = $10,
                default_signer_address = $11,
                updated_at = $12
            WHERE id = $1
            RETURNING id, chain_id, name, rpc_url, other_rpc_urls, test_net,
                      block_explorer_url, fee_multiplier, gas_limit_multiplier,
                      active, default_signer_address, created_at, updated_at
            "#,
        )
        .bind(network.id().as_uuid())
        .bind(network.chain_id())
        .bind(network.name())
        .bind(network.rpc_url())
        .bind(&other_rpc_urls_json)
        .bind(network.test_net())
        .bind(network.block_explorer_url())
        .bind(network.fee_multiplier())
        .bind(network.gas_limit_multiplier())
        .bind(network.active())
        .bind(network.default_signer_address())
        .bind(network.updated_at())
        .fetch_optional(&self.pool)
        .await?;

        row.map(Network::try_from).transpose()
    }

    async fn soft_delete(&self, id: &NetworkId) -> Result<bool, RepositoryError> {
        let result = sqlx::query(
            r#"
            UPDATE networks
            SET active = false, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn exists_by_chain_id(
        &self,
        chain_id: i32,
        exclude_id: Option<&NetworkId>,
    ) -> Result<bool, RepositoryError> {
        let exists = match exclude_id {
            Some(id) => {
                sqlx::query_scalar::<_, bool>(
                    r#"
                    SELECT EXISTS(
                        SELECT 1 FROM networks
                        WHERE chain_id = $1 AND id != $2
                    )
                    "#,
                )
                .bind(chain_id)
                .bind(id.as_uuid())
                .fetch_one(&self.pool)
                .await?
            }
            None => {
                sqlx::query_scalar::<_, bool>(
                    r#"
                    SELECT EXISTS(
                        SELECT 1 FROM networks WHERE chain_id = $1
                    )
                    "#,
                )
                .bind(chain_id)
                .fetch_one(&self.pool)
                .await?
            }
        };

        Ok(exists)
    }
}
