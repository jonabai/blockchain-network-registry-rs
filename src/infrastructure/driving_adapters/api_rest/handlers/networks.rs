//! Network Handlers
//!
//! HTTP handlers for network CRUD operations.
//! All endpoints require JWT authentication.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, patch, post, put},
    Json, Router,
};
use uuid::Uuid;
use validator::Validate;

use crate::domain::models::network::NetworkId;
use crate::infrastructure::driving_adapters::api_rest::dto::network::{
    CreateNetworkDto, NetworkResponseDto, PatchNetworkDto, UpdateNetworkDto,
};
use crate::infrastructure::driving_adapters::api_rest::middleware::auth::JwtAuth;
use crate::infrastructure::driving_adapters::api_rest::AppState;
use crate::shared::errors::ApiError;

/// Create the router for network endpoints
///
/// All routes require JWT authentication via the `JwtAuth` extractor.
/// The `add_config_extension` middleware injects the config into request extensions.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_network))
        .route("/", get(get_active_networks))
        .route("/{id}", get(get_network_by_id))
        .route("/{id}", put(update_network))
        .route("/{id}", patch(partial_update_network))
        .route("/{id}", delete(delete_network))
}

/// POST /networks - Create a new network
///
/// # Authentication
///
/// Requires valid JWT token in Authorization header.
///
/// # Responses
///
/// * 201 Created - Network created successfully
/// * 400 Bad Request - Validation error
/// * 401 Unauthorized - Missing or invalid JWT token
/// * 409 Conflict - Network with same chain_id already exists
#[axum::debug_handler]
async fn create_network(
    _auth: JwtAuth, // Require authentication
    State(state): State<AppState>,
    Json(dto): Json<CreateNetworkDto>,
) -> Result<(StatusCode, Json<NetworkResponseDto>), ApiError> {
    // Validate DTO
    dto.validate()?;

    // Execute use case
    let network = state.create_network_use_case.execute(dto.into()).await?;

    // Return response
    Ok((StatusCode::CREATED, Json(NetworkResponseDto::from(network))))
}

/// GET /networks - Get all active networks
///
/// # Authentication
///
/// Requires valid JWT token in Authorization header.
///
/// # Responses
///
/// * 200 OK - List of active networks (sorted by name)
/// * 401 Unauthorized - Missing or invalid JWT token
#[axum::debug_handler]
async fn get_active_networks(
    _auth: JwtAuth, // Require authentication
    State(state): State<AppState>,
) -> Result<Json<Vec<NetworkResponseDto>>, ApiError> {
    // Execute use case
    let networks = state.get_active_networks_use_case.execute().await?;

    // Return response
    let response: Vec<NetworkResponseDto> = networks.into_iter().map(NetworkResponseDto::from).collect();
    Ok(Json(response))
}

/// GET /networks/:id - Get a network by ID
///
/// # Authentication
///
/// Requires valid JWT token in Authorization header.
///
/// # Responses
///
/// * 200 OK - Network found
/// * 401 Unauthorized - Missing or invalid JWT token
/// * 404 Not Found - Network does not exist
#[axum::debug_handler]
async fn get_network_by_id(
    _auth: JwtAuth, // Require authentication
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<NetworkResponseDto>, ApiError> {
    // Parse UUID
    let uuid = Uuid::parse_str(&id)?;
    let network_id = NetworkId::from_uuid(uuid);

    // Execute use case
    let network = state.get_network_by_id_use_case.execute(&network_id).await?;

    // Return response
    Ok(Json(NetworkResponseDto::from(network)))
}

/// PUT /networks/:id - Full update of a network
///
/// # Authentication
///
/// Requires valid JWT token in Authorization header.
///
/// # Responses
///
/// * 200 OK - Network updated successfully
/// * 400 Bad Request - Validation error
/// * 401 Unauthorized - Missing or invalid JWT token
/// * 404 Not Found - Network does not exist
/// * 409 Conflict - New chain_id already exists
#[axum::debug_handler]
async fn update_network(
    _auth: JwtAuth, // Require authentication
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(dto): Json<UpdateNetworkDto>,
) -> Result<Json<NetworkResponseDto>, ApiError> {
    // Validate DTO
    dto.validate()?;

    // Parse UUID
    let uuid = Uuid::parse_str(&id)?;
    let network_id = NetworkId::from_uuid(uuid);

    // Execute use case
    let network = state
        .update_network_use_case
        .execute(&network_id, dto.into())
        .await?;

    // Return response
    Ok(Json(NetworkResponseDto::from(network)))
}

/// PATCH /networks/:id - Partial update of a network
///
/// # Authentication
///
/// Requires valid JWT token in Authorization header.
///
/// # Responses
///
/// * 200 OK - Network updated successfully
/// * 400 Bad Request - Validation error
/// * 401 Unauthorized - Missing or invalid JWT token
/// * 404 Not Found - Network does not exist
/// * 409 Conflict - New chain_id already exists
#[axum::debug_handler]
async fn partial_update_network(
    _auth: JwtAuth, // Require authentication
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(dto): Json<PatchNetworkDto>,
) -> Result<Json<NetworkResponseDto>, ApiError> {
    // Validate DTO
    dto.validate()?;

    // Parse UUID
    let uuid = Uuid::parse_str(&id)?;
    let network_id = NetworkId::from_uuid(uuid);

    // Execute use case
    let network = state
        .partial_update_network_use_case
        .execute(&network_id, dto.into())
        .await?;

    // Return response
    Ok(Json(NetworkResponseDto::from(network)))
}

/// DELETE /networks/:id - Soft delete a network
///
/// # Authentication
///
/// Requires valid JWT token in Authorization header.
///
/// # Responses
///
/// * 204 No Content - Network deleted successfully
/// * 401 Unauthorized - Missing or invalid JWT token
/// * 404 Not Found - Network does not exist
#[axum::debug_handler]
async fn delete_network(
    _auth: JwtAuth, // Require authentication
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    // Parse UUID
    let uuid = Uuid::parse_str(&id)?;
    let network_id = NetworkId::from_uuid(uuid);

    // Execute use case
    state.delete_network_use_case.execute(&network_id).await?;

    // Return response
    Ok(StatusCode::NO_CONTENT)
}
