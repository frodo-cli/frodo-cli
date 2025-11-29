use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use rand::{rngs::OsRng, RngCore};
use thiserror::Error;

/// Key material used for encryption at rest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyMaterial {
    /// Identifier for logging/rotation (never log key bytes).
    pub id: String,
    /// 256-bit symmetric key.
    pub bytes: [u8; 32],
}

#[derive(Debug, Error)]
pub enum KeyError {
    #[error("keyring error: {0}")]
    Keyring(String),
    #[error("decode error: {0}")]
    Decode(String),
    #[error("generation error: {0}")]
    Generation(String),
}

/// Provides access to encryption keys (OS keychain in production; memory in tests).
#[async_trait]
pub trait KeyProvider: Send + Sync {
    async fn get_or_create(&self) -> Result<KeyMaterial, KeyError>;
}

/// OS keyring-backed provider. Uses the `keyring` crate to store the key.
pub struct KeyringProvider {
    service: String,
    account: String,
}

impl KeyringProvider {
    pub fn new(service: impl Into<String>, account: impl Into<String>) -> Self {
        Self {
            service: service.into(),
            account: account.into(),
        }
    }
}

#[async_trait]
impl KeyProvider for KeyringProvider {
    async fn get_or_create(&self) -> Result<KeyMaterial, KeyError> {
        // Keyring operations are synchronous; wrap in async for trait compatibility.
        match keyring::Entry::new(&self.service, &self.account) {
            Ok(entry) => {
                if let Ok(secret) = entry.get_password() {
                    return decode_key(&secret);
                }

                let material = generate_key();
                entry
                    .set_password(&encode_key(&material))
                    .map_err(|e| KeyError::Keyring(e.to_string()))?;
                Ok(material)
            }
            Err(err) => Err(KeyError::Keyring(err.to_string())),
        }
    }
}

/// In-memory key provider for tests and ephemeral sessions.
#[derive(Debug, Default, Clone)]
pub struct InMemoryKeyProvider {
    inner: Arc<Mutex<Option<KeyMaterial>>>,
}

#[async_trait]
impl KeyProvider for InMemoryKeyProvider {
    async fn get_or_create(&self) -> Result<KeyMaterial, KeyError> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|err| KeyError::Generation(format!("lock poisoned: {err}")))?;

        if let Some(existing) = guard.clone() {
            return Ok(existing);
        }

        let material = generate_key();
        *guard = Some(material.clone());
        Ok(material)
    }
}

fn generate_key() -> KeyMaterial {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    KeyMaterial {
        id: "default".to_string(),
        bytes,
    }
}

fn encode_key(material: &KeyMaterial) -> String {
    general_purpose::STANDARD.encode(material.bytes)
}

fn decode_key(secret: &str) -> Result<KeyMaterial, KeyError> {
    let bytes = general_purpose::STANDARD
        .decode(secret)
        .map_err(|e| KeyError::Decode(e.to_string()))?;

    if bytes.len() != 32 {
        return Err(KeyError::Decode(format!(
            "expected 32 bytes, got {}",
            bytes.len()
        )));
    }

    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(KeyMaterial {
        id: "default".to_string(),
        bytes: out,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn memory_provider_returns_same_key() {
        let provider = InMemoryKeyProvider::default();
        let first = provider.get_or_create().await.unwrap();
        let second = provider.get_or_create().await.unwrap();

        assert_eq!(first.bytes, second.bytes);
        assert_eq!(first.id, second.id);
    }

    #[test]
    fn decode_rejects_wrong_length() {
        let err = decode_key("abcd").expect_err("should reject wrong length");
        assert!(matches!(err, KeyError::Decode(_)));
    }
}
