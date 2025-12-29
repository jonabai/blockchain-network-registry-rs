//! REST API Module
//!
//! Contains HTTP handlers, DTOs, and middleware for the REST API.

pub mod dto;
pub mod handlers;
pub mod middleware;

use std::sync::Arc;

use crate::application::use_cases::networks::{
    CreateNetworkUseCase, DeleteNetworkUseCase, GetActiveNetworksUseCase, GetNetworkByIdUseCase,
    PartialUpdateNetworkUseCase, UpdateNetworkUseCase,
};
use crate::infrastructure::driven_adapters::config::AppConfig;

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub create_network_use_case: Arc<CreateNetworkUseCase>,
    pub get_network_by_id_use_case: Arc<GetNetworkByIdUseCase>,
    pub get_active_networks_use_case: Arc<GetActiveNetworksUseCase>,
    pub update_network_use_case: Arc<UpdateNetworkUseCase>,
    pub partial_update_network_use_case: Arc<PartialUpdateNetworkUseCase>,
    pub delete_network_use_case: Arc<DeleteNetworkUseCase>,
}
