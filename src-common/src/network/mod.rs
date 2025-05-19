// src-common/src/network/mod.rs
// Network connectivity module

pub mod platform;

pub use platform::{NetworkConnectivity, NetworkError, create_network_checker};
