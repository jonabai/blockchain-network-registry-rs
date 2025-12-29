//! Common test utilities for e2e tests
//!
//! Provides test infrastructure for spinning up a PostgreSQL container,
//! running migrations, and creating a test application.

use std::sync::Arc;

use axum::{middleware, Router};
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use testcontainers::{runners::AsyncRunner, ContainerAsync, ImageExt};
use testcontainers_modules::postgres::Postgres;
use tower_http::trace::TraceLayer;

use blockchain_network_registry::application::use_cases::networks::{
    CreateNetworkUseCase, DeleteNetworkUseCase, GetActiveNetworksUseCase, GetNetworkByIdUseCase,
    PartialUpdateNetworkUseCase, UpdateNetworkUseCase,
};
use blockchain_network_registry::infrastructure::driven_adapters::network_repository::PostgresNetworkRepository;
use blockchain_network_registry::infrastructure::driving_adapters::api_rest::handlers::networks;
use blockchain_network_registry::infrastructure::driving_adapters::api_rest::AppState;

/// Test JWT secret (minimum 32 characters)
pub const TEST_JWT_SECRET: &str = "test-jwt-secret-key-for-e2e-testing-only-min-32-chars";

/// JWT claims for test tokens
#[derive(Debug, Serialize, Deserialize)]
pub struct TestClaims {
    pub sub: String,
    pub email: String,
    pub role: String,
    pub iat: i64,
    pub exp: i64,
}

/// Test application context
pub struct TestApp {
    pub router: Router,
    pub pool: PgPool,
    pub jwt_token: String,
    _container: ContainerAsync<Postgres>,
}

impl TestApp {
    /// Create a new test application with a fresh PostgreSQL database
    pub async fn new() -> Self {
        // Start PostgreSQL container
        let container = Postgres::default()
            .with_tag("16-alpine")
            .start()
            .await
            .expect("Failed to start PostgreSQL container");

        let host = container.get_host().await.expect("Failed to get host");
        let port = container
            .get_host_port_ipv4(5432)
            .await
            .expect("Failed to get port");

        let database_url = format!(
            "postgres://postgres:postgres@{}:{}/postgres",
            host, port
        );

        // Create connection pool
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .min_connections(1)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        // Create repository
        let network_repository = Arc::new(PostgresNetworkRepository::new(pool.clone()));

        // Create use cases
        let create_network_use_case = Arc::new(CreateNetworkUseCase::new(network_repository.clone()));
        let get_network_by_id_use_case = Arc::new(GetNetworkByIdUseCase::new(network_repository.clone()));
        let get_active_networks_use_case = Arc::new(GetActiveNetworksUseCase::new(network_repository.clone()));
        let update_network_use_case = Arc::new(UpdateNetworkUseCase::new(network_repository.clone()));
        let partial_update_network_use_case = Arc::new(PartialUpdateNetworkUseCase::new(network_repository.clone()));
        let delete_network_use_case = Arc::new(DeleteNetworkUseCase::new(network_repository.clone()));

        // Create test config (we'll inject it directly into extensions)
        let test_config = create_test_config();
        let config = Arc::new(test_config);

        // Create application state
        let app_state = AppState {
            config: config.clone(),
            create_network_use_case,
            get_network_by_id_use_case,
            get_active_networks_use_case,
            update_network_use_case,
            partial_update_network_use_case,
            delete_network_use_case,
        };

        // Build router (without rate limiting for tests)
        let router = Router::new()
            .nest("/networks", networks::router())
            .layer(middleware::from_fn_with_state(
                app_state.clone(),
                blockchain_network_registry::infrastructure::driving_adapters::api_rest::middleware::auth::add_config_extension,
            ))
            .layer(TraceLayer::new_for_http())
            .with_state(app_state);

        // Generate test JWT token
        let jwt_token = generate_test_token();

        Self {
            router,
            pool,
            jwt_token,
            _container: container,
        }
    }

    /// Get the authorization header value for requests
    pub fn auth_header(&self) -> String {
        format!("Bearer {}", self.jwt_token)
    }

    /// Clear all data from the database (useful between tests)
    pub async fn clear_database(&self) {
        sqlx::query("TRUNCATE TABLE networks CASCADE")
            .execute(&self.pool)
            .await
            .expect("Failed to truncate networks table");
    }
}

/// Create a test configuration
fn create_test_config() -> blockchain_network_registry::infrastructure::driven_adapters::config::AppConfig {
    // We need to deserialize from a config source since AppConfig uses SecretString
    // which requires deserialization. We'll use the config crate with test values.
    use config::{Config, File, FileFormat};

    let config_str = format!(
        r#"
[server]
host = "127.0.0.1"
port = 0
allowed_origins = ["http://localhost:3000"]

[database]
url = "postgres://test:test@localhost/test"
max_connections = 5
min_connections = 1

[jwt]
secret = "{}"
expires_in_secs = 3600

[rate_limit]
requests_per_second = 1000
burst_size = 1000
"#,
        TEST_JWT_SECRET
    );

    Config::builder()
        .add_source(File::from_str(&config_str, FileFormat::Toml))
        .build()
        .expect("Failed to build test config")
        .try_deserialize()
        .expect("Failed to deserialize test config")
}

/// Generate a valid JWT token for testing
pub fn generate_test_token() -> String {
    let now = Utc::now().timestamp();
    let claims = TestClaims {
        sub: "test-user-id".to_string(),
        email: "test@example.com".to_string(),
        role: "admin".to_string(),
        iat: now,
        exp: now + 3600, // 1 hour from now
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(TEST_JWT_SECRET.as_bytes()),
    )
    .expect("Failed to generate test JWT token")
}

/// Generate an expired JWT token for testing unauthorized scenarios
pub fn generate_expired_token() -> String {
    let now = Utc::now().timestamp();
    let claims = TestClaims {
        sub: "test-user-id".to_string(),
        email: "test@example.com".to_string(),
        role: "admin".to_string(),
        iat: now - 7200, // 2 hours ago
        exp: now - 3600, // 1 hour ago (expired)
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(TEST_JWT_SECRET.as_bytes()),
    )
    .expect("Failed to generate expired JWT token")
}

/// Helper struct for creating network request bodies
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateNetworkRequest {
    pub chain_id: i32,
    pub name: String,
    pub rpc_url: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub other_rpc_urls: Vec<String>,
    pub test_net: bool,
    pub block_explorer_url: String,
    pub fee_multiplier: f64,
    pub gas_limit_multiplier: f64,
    pub default_signer_address: String,
}

impl Default for CreateNetworkRequest {
    fn default() -> Self {
        Self {
            chain_id: 1,
            name: "Ethereum Mainnet".to_string(),
            rpc_url: "https://mainnet.infura.io/v3/test".to_string(),
            other_rpc_urls: vec![],
            test_net: false,
            block_explorer_url: "https://etherscan.io".to_string(),
            fee_multiplier: 1.0,
            gas_limit_multiplier: 1.2,
            default_signer_address: "0x742d35Cc6634C0532925a3b844Bc9e7595f1dEaD".to_string(),
        }
    }
}

impl CreateNetworkRequest {
    pub fn with_chain_id(mut self, chain_id: i32) -> Self {
        self.chain_id = chain_id;
        self
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }
}

/// Helper struct for updating network request bodies
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNetworkRequest {
    pub chain_id: i32,
    pub name: String,
    pub rpc_url: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub other_rpc_urls: Vec<String>,
    pub test_net: bool,
    pub block_explorer_url: String,
    pub fee_multiplier: f64,
    pub gas_limit_multiplier: f64,
    pub default_signer_address: String,
}

impl Default for UpdateNetworkRequest {
    fn default() -> Self {
        Self {
            chain_id: 1,
            name: "Updated Network".to_string(),
            rpc_url: "https://updated.rpc.url".to_string(),
            other_rpc_urls: vec![],
            test_net: false,
            block_explorer_url: "https://updated.explorer.io".to_string(),
            fee_multiplier: 1.5,
            gas_limit_multiplier: 1.3,
            default_signer_address: "0x742d35Cc6634C0532925a3b844Bc9e7595f1dEaD".to_string(),
        }
    }
}

/// Helper struct for partial update request bodies
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchNetworkRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rpc_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub other_rpc_urls: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_net: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_explorer_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_multiplier: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_limit_multiplier: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_signer_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
}

/// Network response structure for deserialization
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct NetworkResponse {
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
    pub created_at: String,
    pub updated_at: String,
}

/// Error response structure for deserialization
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
    pub request_id: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    pub details: Option<Vec<FieldError>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}
