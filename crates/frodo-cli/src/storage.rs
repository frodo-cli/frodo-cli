use std::path::PathBuf;

use crate::config::Config;
use color_eyre::Result;
use dirs::data_dir;
use frodo_storage::{
    key_provider::{InMemoryKeyProvider, KeyringProvider},
    secure_file_store::EncryptedFileStore,
};
use tracing::debug;

/// Resolve the default data directory for Frodo.
pub fn default_data_dir() -> Result<PathBuf> {
    let base = data_dir().ok_or_else(|| color_eyre::eyre::eyre!("no data dir available"))?;
    Ok(base.join("frodo"))
}

/// Build a production-grade encrypted store using the OS keychain.
pub fn production_store() -> Result<EncryptedFileStore<KeyringProvider>> {
    let root = default_data_dir()?;
    debug!(?root, "initializing encrypted store");
    Ok(EncryptedFileStore::new(
        root,
        KeyringProvider::new("frodo-cli", "data-key"),
    ))
}

/// Build a store using config overrides.
pub fn store_from_config(config: &Config) -> Result<EncryptedFileStore<KeyringProvider>> {
    if let Some(root) = &config.data_dir {
        debug!(?root, "initializing encrypted store (config override)");
        return Ok(EncryptedFileStore::new(
            root.clone(),
            KeyringProvider::new("frodo-cli", "data-key"),
        ));
    }

    production_store()
}

/// Helper for tests to construct a store rooted at a temp dir with an in-memory key.
#[cfg(test)]
pub fn test_store(root: impl Into<PathBuf>) -> EncryptedFileStore<InMemoryKeyProvider> {
    EncryptedFileStore::new(root, InMemoryKeyProvider::default())
}
