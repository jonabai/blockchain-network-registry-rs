//! Use Cases
//!
//! Application-specific business rules.
//! Each use case is a single-purpose struct with an execute() method.

pub mod networks;

pub use networks::{
    CreateNetworkUseCase, DeleteNetworkUseCase, GetActiveNetworksUseCase, GetNetworkByIdUseCase,
    PartialUpdateNetworkUseCase, UpdateNetworkUseCase,
};
