use color_eyre::Result;
use frodo_sync::{NoopSync, TaskSync};
use tracing::info;

use crate::config;

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
    provider
        .push(&[])
        .await
        .map_err(|e| color_eyre::eyre::eyre!(e.to_string()))?;
    Ok(())
}
