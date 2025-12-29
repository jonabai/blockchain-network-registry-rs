//! Data Transfer Objects
//!
//! Request and response DTOs for the REST API.

pub mod network;

pub use network::{
    CreateNetworkDto, NetworkResponseDto, PatchNetworkDto, UpdateNetworkDto,
};
