use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use async_trait::async_trait;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use frodo_core::storage::{SecureStore, SecureStoreError};
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;
use tracing::instrument;

use crate::key_provider::{KeyMaterial, KeyProvider};

/// AES-GCM encrypted file-backed store implementing the shared `SecureStore` contract.
/// Keys are persisted via a `KeyProvider` (OS keyring in production).
pub struct EncryptedFileStore<P: KeyProvider> {
    root: PathBuf,
    key_provider: P,
}

impl<P: KeyProvider> EncryptedFileStore<P> {
    pub fn new(root: impl Into<PathBuf>, key_provider: P) -> Self {
        Self {
            root: root.into(),
            key_provider,
        }
    }

    fn path_for(&self, key: &str) -> PathBuf {
        self.root.join(sanitize_key(key))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct StoredBlob {
    nonce: String,
    ciphertext: String,
}

#[async_trait]
impl<P: KeyProvider> SecureStore for EncryptedFileStore<P> {
    #[instrument(skip_all, fields(key))]
    async fn put(&self, key: &str, value: &[u8]) -> Result<(), SecureStoreError> {
        fs::create_dir_all(&self.root).map_err(storage_err)?;

        let key_material =
            self.key_provider
                .get_or_create()
                .await
                .map_err(|e| SecureStoreError::Storage {
                    reason: format!("key provider: {e}"),
                })?;

        let cipher = build_cipher(&key_material)?;
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = cipher
            .encrypt(&nonce, value)
            .map_err(|e| SecureStoreError::Storage {
                reason: format!("encrypt failed: {e}"),
            })?;

        let blob = StoredBlob {
            nonce: URL_SAFE_NO_PAD.encode(nonce.as_slice()),
            ciphertext: URL_SAFE_NO_PAD.encode(ciphertext),
        };

        let path = self.path_for(key);
        write_blob(&path, &blob)
    }

    #[instrument(skip_all, fields(key))]
    async fn get(&self, key: &str) -> Result<Vec<u8>, SecureStoreError> {
        let path = self.path_for(key);
        let blob = read_blob(&path)?;

        let key_material =
            self.key_provider
                .get_or_create()
                .await
                .map_err(|e| SecureStoreError::Storage {
                    reason: format!("key provider: {e}"),
                })?;
        let cipher = build_cipher(&key_material)?;

        let nonce_bytes =
            URL_SAFE_NO_PAD
                .decode(blob.nonce)
                .map_err(|e| SecureStoreError::Storage {
                    reason: format!("nonce decode failed: {e}"),
                })?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext =
            URL_SAFE_NO_PAD
                .decode(blob.ciphertext)
                .map_err(|e| SecureStoreError::Storage {
                    reason: format!("ciphertext decode failed: {e}"),
                })?;

        cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| SecureStoreError::Storage {
                reason: format!("decrypt failed: {e}"),
            })
    }

    #[instrument(skip_all, fields(key))]
    async fn delete(&self, key: &str) -> Result<(), SecureStoreError> {
        let path = self.path_for(key);
        match fs::remove_file(path) {
            Ok(_) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(storage_err(err)),
        }
    }
}

fn write_blob(path: &Path, blob: &StoredBlob) -> Result<(), SecureStoreError> {
    let parent = path.parent().ok_or_else(|| SecureStoreError::Storage {
        reason: "invalid storage path".to_string(),
    })?;
    fs::create_dir_all(parent).map_err(storage_err)?;

    let mut tmp = NamedTempFile::new_in(parent).map_err(storage_err)?;
    let json = serde_json::to_vec(blob).map_err(storage_err)?;
    tmp.write_all(&json).map_err(storage_err)?;
    tmp.flush().map_err(storage_err)?;
    tmp.persist(path).map_err(|e| storage_err(e.error))?;
    Ok(())
}

fn read_blob(path: &Path) -> Result<StoredBlob, SecureStoreError> {
    let mut file = File::open(path).map_err(|err| {
        if err.kind() == std::io::ErrorKind::NotFound {
            SecureStoreError::NotFound {
                key: path.to_string_lossy().to_string(),
            }
        } else {
            storage_err(err)
        }
    })?;

    let mut buf = Vec::new();
    file.read_to_end(&mut buf).map_err(storage_err)?;
    serde_json::from_slice(&buf).map_err(storage_err)
}

fn build_cipher(material: &KeyMaterial) -> Result<Aes256Gcm, SecureStoreError> {
    Aes256Gcm::new_from_slice(&material.bytes).map_err(|e| SecureStoreError::Storage {
        reason: format!("cipher init failed: {e}"),
    })
}

fn sanitize_key(key: &str) -> String {
    URL_SAFE_NO_PAD.encode(key)
}

fn storage_err<E: ToString>(err: E) -> SecureStoreError {
    SecureStoreError::Storage {
        reason: err.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use frodo_core::storage::SecureStore;

    use super::*;
    use crate::key_provider::InMemoryKeyProvider;

    #[tokio::test]
    async fn round_trip_encrypts_and_decrypts() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = EncryptedFileStore::new(dir.path(), InMemoryKeyProvider::default());

        let key = "workspace/session";
        let value = b"hello-frodo";

        store.put(key, value).await.expect("put");
        let decrypted = store.get(key).await.expect("get");
        assert_eq!(decrypted, value);

        // ensure plaintext is not present on disk
        let stored = std::fs::read_to_string(store.path_for(key)).expect("read ciphertext");
        assert!(
            !stored.contains("hello-frodo"),
            "plaintext must not be stored"
        );
    }

    #[tokio::test]
    async fn delete_is_idempotent() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = EncryptedFileStore::new(dir.path(), InMemoryKeyProvider::default());
        let key = "k";
        store.put(key, b"v").await.expect("put");
        store.delete(key).await.expect("delete");
        store.delete(key).await.expect("delete again");

        let err = store.get(key).await.expect_err("should be missing");
        assert!(matches!(err, SecureStoreError::NotFound { .. }));
    }
}
