use color_eyre::Result;
use frodo_core::tasks::TaskRepository;
use frodo_storage::secure_file_store::EncryptedFileStore;
use frodo_sync::{NoopSync, TaskSync};
use frodo_task::SecureStoreTaskRepo;
use tracing::info;

use crate::config;
use crate::storage;

/// Placeholder sync handler. Uses a no-op sync provider for now.
pub async fn run(_config: &config::Config) -> Result<()> {
    let provider = NoopSync;
    info!("sync invoked (provider: {})", provider.name());
    println!("Sync is not yet implemented. Planned targets:");
    println!("- Jira: configure project/site and token (todo)");
    println!("- GitHub Issues: derive from git remotes and token (todo)");
    // Stub pull/push
    let remote = provider
        .pull()
        .await
        .map_err(|e| color_eyre::eyre::eyre!(e.to_string()))?;
    println!("Pulled {} remote tasks.", remote.len());
    // Load local tasks to show the counts we would push.
    let store: EncryptedFileStore<_> = storage::store_from_config(_config)?;
    let repo: SecureStoreTaskRepo<_> = SecureStoreTaskRepo::new(store);
    let local = repo
        .list()
        .await
        .map_err(|e| color_eyre::eyre::eyre!(e.to_string()))?;
    println!("Local tasks: {}", local.len());
    provider
        .push(&[])
        .await
        .map_err(|e| color_eyre::eyre::eyre!(e.to_string()))?;
    Ok(())
}
