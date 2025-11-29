use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use thiserror::Error;

/// Errors produced by secure storage implementations.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SecureStoreError {
    /// Requested key does not exist.
    #[error("entry not found for key: {key}")]
    NotFound { key: String },
    /// Underlying storage failure.
    #[error("storage failure: {reason}")]
    Storage { reason: String },
}

/// Simple contract for encrypted-at-rest storage used by agents, tasks, and conversations.
#[async_trait]
pub trait SecureStore: Send + Sync {
    /// Persist a value under a key, overwriting any existing entry.
    async fn put(&self, key: &str, value: &[u8]) -> Result<(), SecureStoreError>;

    /// Retrieve the value for a key.
    async fn get(&self, key: &str) -> Result<Vec<u8>, SecureStoreError>;

    /// Remove a key and its value (idempotent).
    async fn delete(&self, key: &str) -> Result<(), SecureStoreError>;
}

/// In-memory secure store that simulates encryption for tests and smoke runs.
/// This is not cryptographically secure; production implementations must use
/// SQLCipher or AES-GCM with keys wrapped by the OS keychain.
#[derive(Debug, Default, Clone)]
pub struct InMemorySecureStore {
    inner: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl InMemorySecureStore {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl SecureStore for InMemorySecureStore {
    async fn put(&self, key: &str, value: &[u8]) -> Result<(), SecureStoreError> {
        let mut map = self.inner.lock().map_err(|err| SecureStoreError::Storage {
            reason: format!("lock poisoned: {err}"),
        })?;

        // XOR is a placeholder to avoid storing plaintext in tests.
        let masked = mask(value);
        map.insert(key.to_string(), masked);
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Vec<u8>, SecureStoreError> {
        let map = self.inner.lock().map_err(|err| SecureStoreError::Storage {
            reason: format!("lock poisoned: {err}"),
        })?;

        let masked = map
            .get(key)
            .cloned()
            .ok_or_else(|| SecureStoreError::NotFound {
                key: key.to_string(),
            })?;
        Ok(unmask(&masked))
    }

    async fn delete(&self, key: &str) -> Result<(), SecureStoreError> {
        let mut map = self.inner.lock().map_err(|err| SecureStoreError::Storage {
            reason: format!("lock poisoned: {err}"),
        })?;
        map.remove(key);
        Ok(())
    }
}

const MASK_BYTE: u8 = 0xA5;

fn mask(input: &[u8]) -> Vec<u8> {
    input.iter().map(|b| b ^ MASK_BYTE).collect()
}

fn unmask(input: &[u8]) -> Vec<u8> {
    mask(input) // XOR twice restores original.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn round_trip_masks_and_unmasks() {
        let store = InMemorySecureStore::new();
        let key = "agent/session";
        let secret = b"top-secret-payload";

        store.put(key, secret).await.expect("put should succeed");
        let retrieved = store.get(key).await.expect("get should succeed");

        assert_eq!(retrieved, secret);
        // Ensure we are not storing plaintext.
        let inner = store.inner.lock().expect("lock");
        assert_ne!(inner.get(key).unwrap(), &secret.to_vec());
    }

    #[tokio::test]
    async fn delete_is_idempotent_and_removes_data() {
        let store = InMemorySecureStore::new();
        store.put("k", b"v").await.expect("put should succeed");
        store.delete("k").await.expect("delete should succeed");
        store
            .delete("k")
            .await
            .expect("delete again should still succeed");

        let err = store
            .get("k")
            .await
            .expect_err("get should fail after delete");
        assert!(matches!(err, SecureStoreError::NotFound { .. }));
    }
}
