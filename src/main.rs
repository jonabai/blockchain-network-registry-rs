//! Blockchain Network Registry API - Main Entry Point

use std::{net::SocketAddr, sync::Arc};

use axum::{middleware, Router};
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use blockchain_network_registry::application::use_cases::networks::{
    CreateNetworkUseCase, DeleteNetworkUseCase, GetActiveNetworksUseCase, GetNetworkByIdUseCase,
    PartialUpdateNetworkUseCase, UpdateNetworkUseCase,
};
use blockchain_network_registry::infrastructure::driven_adapters::config::AppConfig;
use blockchain_network_registry::infrastructure::driven_adapters::network_repository::PostgresNetworkRepository;
use blockchain_network_registry::infrastructure::driving_adapters::api_rest::handlers::networks;
use blockchain_network_registry::infrastructure::driving_adapters::api_rest::middleware::auth::add_config_extension;
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

    // Configure rate limiting
    let governor_conf = GovernorConfigBuilder::default()
        .per_second(config.rate_limit.requests_per_second.into())
        .burst_size(config.rate_limit.burst_size)
        .finish()
        .expect("Failed to build rate limiter config");

    let rate_limit_layer = GovernorLayer {
        config: Arc::new(governor_conf),
    };
    tracing::info!(
        "Rate limiting configured: {} req/s, burst: {}",
        config.rate_limit.requests_per_second,
        config.rate_limit.burst_size
    );

    // Build router with secure CORS configuration
    let cors = if config.server.allowed_origins.is_empty() {
        // Development: restrictive default (localhost only)
        tracing::warn!("No allowed_origins configured, defaulting to localhost only");
        CorsLayer::new()
            .allow_origin("http://localhost:3000".parse::<axum::http::HeaderValue>().expect("valid origin"))
            .allow_methods([axum::http::Method::GET, axum::http::Method::POST, axum::http::Method::PUT, axum::http::Method::PATCH, axum::http::Method::DELETE])
            .allow_headers([axum::http::header::CONTENT_TYPE, axum::http::header::AUTHORIZATION])
            .allow_credentials(true)
    } else {
        let origins: Vec<axum::http::HeaderValue> = config
            .server
            .allowed_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods([axum::http::Method::GET, axum::http::Method::POST, axum::http::Method::PUT, axum::http::Method::PATCH, axum::http::Method::DELETE])
            .allow_headers([axum::http::header::CONTENT_TYPE, axum::http::header::AUTHORIZATION])
            .allow_credentials(true)
    };

    let app = Router::new()
        .nest("/networks", networks::router())
        // Add config to request extensions for JWT validation
        .layer(middleware::from_fn_with_state(app_state.clone(), add_config_extension))
        .layer(rate_limit_layer)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(app_state);

    // Start server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!("Server listening on {}", addr);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
