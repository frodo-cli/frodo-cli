//! Concrete storage implementations with encryption at rest.
//! Uses AES-GCM with keys sourced from the OS keyring (or test doubles).

pub mod key_provider;
pub mod secure_file_store;
