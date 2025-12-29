//! End-to-end tests for network endpoints
//!
//! These tests spin up a real PostgreSQL database using testcontainers,
//! run migrations, and test all network CRUD endpoints.

mod common;

use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
};
use tower::util::ServiceExt;

use common::{
    CreateNetworkRequest, ErrorResponse, NetworkResponse, PatchNetworkRequest, TestApp,
    UpdateNetworkRequest,
};

// ============================================================================
// POST /networks - Create Network Tests
// ============================================================================

#[tokio::test]
async fn test_create_network_success() {
    let app = TestApp::new().await;

    let request_body = CreateNetworkRequest::default();

    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/networks")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, app.auth_header())
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let network: NetworkResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(network.chain_id, request_body.chain_id);
    assert_eq!(network.name, request_body.name);
    assert_eq!(network.rpc_url, request_body.rpc_url);
    assert!(network.active);
}

#[tokio::test]
async fn test_create_network_with_other_rpc_urls() {
    let app = TestApp::new().await;

    let mut request_body = CreateNetworkRequest::default().with_chain_id(137);
    request_body.name = "Polygon".to_string();
    request_body.other_rpc_urls = vec![
        "https://polygon-rpc.com".to_string(),
        "https://rpc-mainnet.maticvigil.com".to_string(),
    ];

    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/networks")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, app.auth_header())
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let network: NetworkResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(network.other_rpc_urls.len(), 2);
}

#[tokio::test]
async fn test_create_network_duplicate_chain_id_returns_conflict() {
    let app = TestApp::new().await;

    let request_body = CreateNetworkRequest::default();

    // Create first network
    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/networks")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, app.auth_header())
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // Try to create second network with same chain_id
    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/networks")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, app.auth_header())
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let error: ErrorResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(error.error.code, "CONFLICT");
}

#[tokio::test]
async fn test_create_network_invalid_chain_id_returns_bad_request() {
    let app = TestApp::new().await;

    let mut request_body = CreateNetworkRequest::default();
    request_body.chain_id = 0; // Invalid: must be >= 1

    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/networks")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, app.auth_header())
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_network_invalid_ethereum_address_returns_bad_request() {
    let app = TestApp::new().await;

    let mut request_body = CreateNetworkRequest::default();
    request_body.default_signer_address = "invalid-address".to_string();

    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/networks")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, app.auth_header())
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_network_without_auth_returns_unauthorized() {
    let app = TestApp::new().await;

    let request_body = CreateNetworkRequest::default();

    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/networks")
                .header(header::CONTENT_TYPE, "application/json")
                // No Authorization header
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_create_network_with_expired_token_returns_unauthorized() {
    let app = TestApp::new().await;

    let request_body = CreateNetworkRequest::default();
    let expired_token = common::generate_expired_token();

    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/networks")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, format!("Bearer {}", expired_token))
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ============================================================================
// GET /networks - Get Active Networks Tests
// ============================================================================

#[tokio::test]
async fn test_get_active_networks_empty() {
    let app = TestApp::new().await;

    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/networks")
                .header(header::AUTHORIZATION, app.auth_header())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let networks: Vec<NetworkResponse> = serde_json::from_slice(&body).unwrap();

    assert!(networks.is_empty());
}

#[tokio::test]
async fn test_get_active_networks_returns_only_active() {
    let app = TestApp::new().await;

    // Create two networks
    let network1 = CreateNetworkRequest::default()
        .with_chain_id(1)
        .with_name("Ethereum");
    let network2 = CreateNetworkRequest::default()
        .with_chain_id(137)
        .with_name("Polygon");

    for network in [&network1, &network2] {
        app.router
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/networks")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::AUTHORIZATION, app.auth_header())
                    .body(Body::from(serde_json::to_string(network).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
    }

    // Get all active networks
    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/networks")
                .header(header::AUTHORIZATION, app.auth_header())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let networks: Vec<NetworkResponse> = serde_json::from_slice(&body).unwrap();

    assert_eq!(networks.len(), 2);
    // Should be sorted by name
    assert_eq!(networks[0].name, "Ethereum");
    assert_eq!(networks[1].name, "Polygon");
}

#[tokio::test]
async fn test_get_active_networks_without_auth_returns_unauthorized() {
    let app = TestApp::new().await;

    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/networks")
                // No Authorization header
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ============================================================================
// GET /networks/:id - Get Network by ID Tests
// ============================================================================

#[tokio::test]
async fn test_get_network_by_id_success() {
    let app = TestApp::new().await;

    // Create a network
    let request_body = CreateNetworkRequest::default();
    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/networks")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, app.auth_header())
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let created: NetworkResponse = serde_json::from_slice(&body).unwrap();

    // Get the network by ID
    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/networks/{}", created.id))
                .header(header::AUTHORIZATION, app.auth_header())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let network: NetworkResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(network.id, created.id);
    assert_eq!(network.chain_id, request_body.chain_id);
}

#[tokio::test]
async fn test_get_network_by_id_not_found() {
    let app = TestApp::new().await;

    let fake_id = "550e8400-e29b-41d4-a716-446655440000";

    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/networks/{}", fake_id))
                .header(header::AUTHORIZATION, app.auth_header())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_network_by_id_invalid_uuid() {
    let app = TestApp::new().await;

    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/networks/invalid-uuid")
                .header(header::AUTHORIZATION, app.auth_header())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// PUT /networks/:id - Update Network Tests
// ============================================================================

#[tokio::test]
async fn test_update_network_success() {
    let app = TestApp::new().await;

    // Create a network
    let create_body = CreateNetworkRequest::default();
    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/networks")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, app.auth_header())
                .body(Body::from(serde_json::to_string(&create_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let created: NetworkResponse = serde_json::from_slice(&body).unwrap();

    // Update the network
    let update_body = UpdateNetworkRequest {
        chain_id: 1,
        name: "Updated Ethereum".to_string(),
        rpc_url: "https://new-rpc.infura.io".to_string(),
        other_rpc_urls: vec!["https://backup.rpc.io".to_string()],
        test_net: true,
        block_explorer_url: "https://new-etherscan.io".to_string(),
        fee_multiplier: 2.0,
        gas_limit_multiplier: 1.5,
        default_signer_address: "0x742d35Cc6634C0532925a3b844Bc9e7595f1dEaD".to_string(),
    };

    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri(format!("/networks/{}", created.id))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, app.auth_header())
                .body(Body::from(serde_json::to_string(&update_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let updated: NetworkResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(updated.name, "Updated Ethereum");
    assert_eq!(updated.rpc_url, "https://new-rpc.infura.io");
    assert!(updated.test_net);
    assert_eq!(updated.fee_multiplier, 2.0);
}

#[tokio::test]
async fn test_update_network_not_found() {
    let app = TestApp::new().await;

    let fake_id = "550e8400-e29b-41d4-a716-446655440000";
    let update_body = UpdateNetworkRequest::default();

    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri(format!("/networks/{}", fake_id))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, app.auth_header())
                .body(Body::from(serde_json::to_string(&update_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_update_network_chain_id_conflict() {
    let app = TestApp::new().await;

    // Create two networks
    let network1 = CreateNetworkRequest::default().with_chain_id(1);
    let network2 = CreateNetworkRequest::default()
        .with_chain_id(137)
        .with_name("Polygon");

    app.router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/networks")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, app.auth_header())
                .body(Body::from(serde_json::to_string(&network1).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/networks")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, app.auth_header())
                .body(Body::from(serde_json::to_string(&network2).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let created2: NetworkResponse = serde_json::from_slice(&body).unwrap();

    // Try to update network2 with network1's chain_id
    let mut update_body = UpdateNetworkRequest::default();
    update_body.chain_id = 1; // Conflict!

    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri(format!("/networks/{}", created2.id))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, app.auth_header())
                .body(Body::from(serde_json::to_string(&update_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

// ============================================================================
// PATCH /networks/:id - Partial Update Network Tests
// ============================================================================

#[tokio::test]
async fn test_patch_network_single_field() {
    let app = TestApp::new().await;

    // Create a network
    let create_body = CreateNetworkRequest::default();
    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/networks")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, app.auth_header())
                .body(Body::from(serde_json::to_string(&create_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let created: NetworkResponse = serde_json::from_slice(&body).unwrap();

    // Patch only the name
    let patch_body = PatchNetworkRequest {
        name: Some("Patched Ethereum".to_string()),
        ..Default::default()
    };

    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri(format!("/networks/{}", created.id))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, app.auth_header())
                .body(Body::from(serde_json::to_string(&patch_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let patched: NetworkResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(patched.name, "Patched Ethereum");
    // Other fields should remain unchanged
    assert_eq!(patched.chain_id, create_body.chain_id);
    assert_eq!(patched.rpc_url, create_body.rpc_url);
}

#[tokio::test]
async fn test_patch_network_deactivate() {
    let app = TestApp::new().await;

    // Create a network
    let create_body = CreateNetworkRequest::default();
    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/networks")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, app.auth_header())
                .body(Body::from(serde_json::to_string(&create_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let created: NetworkResponse = serde_json::from_slice(&body).unwrap();
    assert!(created.active);

    // Deactivate via PATCH
    let patch_body = PatchNetworkRequest {
        active: Some(false),
        ..Default::default()
    };

    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri(format!("/networks/{}", created.id))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, app.auth_header())
                .body(Body::from(serde_json::to_string(&patch_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let patched: NetworkResponse = serde_json::from_slice(&body).unwrap();

    assert!(!patched.active);
}

#[tokio::test]
async fn test_patch_network_empty_body_succeeds() {
    let app = TestApp::new().await;

    // Create a network
    let create_body = CreateNetworkRequest::default();
    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/networks")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, app.auth_header())
                .body(Body::from(serde_json::to_string(&create_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let created: NetworkResponse = serde_json::from_slice(&body).unwrap();

    // Send empty patch body
    let patch_body = PatchNetworkRequest::default();

    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri(format!("/networks/{}", created.id))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, app.auth_header())
                .body(Body::from(serde_json::to_string(&patch_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_patch_network_invalid_url_returns_bad_request() {
    let app = TestApp::new().await;

    // Create a network
    let create_body = CreateNetworkRequest::default();
    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/networks")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, app.auth_header())
                .body(Body::from(serde_json::to_string(&create_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let created: NetworkResponse = serde_json::from_slice(&body).unwrap();

    // Try to patch with invalid URL
    let patch_body = PatchNetworkRequest {
        rpc_url: Some("invalid-url".to_string()),
        ..Default::default()
    };

    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri(format!("/networks/{}", created.id))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, app.auth_header())
                .body(Body::from(serde_json::to_string(&patch_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// ============================================================================
// DELETE /networks/:id - Delete Network Tests
// ============================================================================

#[tokio::test]
async fn test_delete_network_success() {
    let app = TestApp::new().await;

    // Create a network
    let create_body = CreateNetworkRequest::default();
    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/networks")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, app.auth_header())
                .body(Body::from(serde_json::to_string(&create_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let created: NetworkResponse = serde_json::from_slice(&body).unwrap();

    // Delete the network
    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri(format!("/networks/{}", created.id))
                .header(header::AUTHORIZATION, app.auth_header())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Verify network is soft-deleted (should not appear in active list)
    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/networks")
                .header(header::AUTHORIZATION, app.auth_header())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let networks: Vec<NetworkResponse> = serde_json::from_slice(&body).unwrap();

    assert!(networks.is_empty());
}

#[tokio::test]
async fn test_delete_network_not_found() {
    let app = TestApp::new().await;

    let fake_id = "550e8400-e29b-41d4-a716-446655440000";

    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri(format!("/networks/{}", fake_id))
                .header(header::AUTHORIZATION, app.auth_header())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_network_without_auth_returns_unauthorized() {
    let app = TestApp::new().await;

    let fake_id = "550e8400-e29b-41d4-a716-446655440000";

    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri(format!("/networks/{}", fake_id))
                // No Authorization header
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ============================================================================
// Integration Scenarios
// ============================================================================

#[tokio::test]
async fn test_full_crud_lifecycle() {
    let app = TestApp::new().await;

    // 1. Create a network
    let create_body = CreateNetworkRequest::default()
        .with_chain_id(42161)
        .with_name("Arbitrum One");

    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/networks")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, app.auth_header())
                .body(Body::from(serde_json::to_string(&create_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let created: NetworkResponse = serde_json::from_slice(&body).unwrap();
    let network_id = created.id.clone();

    // 2. Read the network
    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/networks/{}", network_id))
                .header(header::AUTHORIZATION, app.auth_header())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 3. Update the network (full update)
    let update_body = UpdateNetworkRequest {
        chain_id: 42161,
        name: "Arbitrum One Updated".to_string(),
        rpc_url: "https://arb1.arbitrum.io/rpc".to_string(),
        other_rpc_urls: vec![],
        test_net: false,
        block_explorer_url: "https://arbiscan.io".to_string(),
        fee_multiplier: 1.1,
        gas_limit_multiplier: 1.2,
        default_signer_address: "0x742d35Cc6634C0532925a3b844Bc9e7595f1dEaD".to_string(),
    };

    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri(format!("/networks/{}", network_id))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, app.auth_header())
                .body(Body::from(serde_json::to_string(&update_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 4. Partial update
    let patch_body = PatchNetworkRequest {
        name: Some("Arbitrum One Final".to_string()),
        ..Default::default()
    };

    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri(format!("/networks/{}", network_id))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, app.auth_header())
                .body(Body::from(serde_json::to_string(&patch_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let final_network: NetworkResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(final_network.name, "Arbitrum One Final");

    // 5. Delete the network
    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri(format!("/networks/{}", network_id))
                .header(header::AUTHORIZATION, app.auth_header())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // 6. Verify it's gone from active list
    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/networks")
                .header(header::AUTHORIZATION, app.auth_header())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let networks: Vec<NetworkResponse> = serde_json::from_slice(&body).unwrap();
    assert!(networks.is_empty());
}

#[tokio::test]
async fn test_create_multiple_networks_with_different_chain_ids() {
    let app = TestApp::new().await;

    let chains = vec![
        (1, "Ethereum"),
        (137, "Polygon"),
        (42161, "Arbitrum"),
        (10, "Optimism"),
        (56, "BSC"),
    ];

    for (chain_id, name) in &chains {
        let body = CreateNetworkRequest::default()
            .with_chain_id(*chain_id)
            .with_name(name);

        let response = app
            .router
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/networks")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::AUTHORIZATION, app.auth_header())
                    .body(Body::from(serde_json::to_string(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::CREATED,
            "Failed to create network: {}",
            name
        );
    }

    // Verify all networks were created
    let response = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/networks")
                .header(header::AUTHORIZATION, app.auth_header())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let networks: Vec<NetworkResponse> = serde_json::from_slice(&body).unwrap();

    assert_eq!(networks.len(), 5);
}
