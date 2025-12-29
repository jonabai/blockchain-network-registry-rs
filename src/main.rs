//! Blockchain Network Registry API - Main Entry Point

use std::sync::Arc;

use axum::Router;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use blockchain_network_registry::application::use_cases::networks::{
    CreateNetworkUseCase, DeleteNetworkUseCase, GetActiveNetworksUseCase, GetNetworkByIdUseCase,
    PartialUpdateNetworkUseCase, UpdateNetworkUseCase,
};
use blockchain_network_registry::infrastructure::driven_adapters::config::AppConfig;
use blockchain_network_registry::infrastructure::driven_adapters::network_repository::PostgresNetworkRepository;
use blockchain_network_registry::infrastructure::driving_adapters::api_rest::handlers::networks;
use blockchain_network_registry::infrastructure::driving_adapters::api_rest::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "blockchain_network_registry=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = AppConfig::load()?;
    tracing::info!("Configuration loaded successfully");

    // Create database connection pool
    let pool = PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .min_connections(config.database.min_connections)
        .connect(&config.database.url)
        .await?;
    tracing::info!("Database connection pool created");

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("Database migrations completed");

    // Create repository
    let network_repository = Arc::new(PostgresNetworkRepository::new(pool));

    // Create use cases
    let create_network_use_case = Arc::new(CreateNetworkUseCase::new(network_repository.clone()));
    let get_network_by_id_use_case = Arc::new(GetNetworkByIdUseCase::new(network_repository.clone()));
    let get_active_networks_use_case = Arc::new(GetActiveNetworksUseCase::new(network_repository.clone()));
    let update_network_use_case = Arc::new(UpdateNetworkUseCase::new(network_repository.clone()));
    let partial_update_network_use_case = Arc::new(PartialUpdateNetworkUseCase::new(network_repository.clone()));
    let delete_network_use_case = Arc::new(DeleteNetworkUseCase::new(network_repository.clone()));

    // Create application state
    let app_state = AppState {
        config: Arc::new(config.clone()),
        create_network_use_case,
        get_network_by_id_use_case,
        get_active_networks_use_case,
        update_network_use_case,
        partial_update_network_use_case,
        delete_network_use_case,
    };

    // Build router
    let app = Router::new()
        .nest("/networks", networks::router())
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(app_state);

    // Start server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!("Server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
