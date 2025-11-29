use color_eyre::Result;
use tracing::info;

use crate::config;

/// Placeholder sync handler. Wires to config and prints what would be synced.
pub async fn run(_config: &config::Config) -> Result<()> {
    info!("sync invoked (stub)");
    println!("Sync is not yet implemented. Planned targets:");
    println!("- Jira: configure project/site and token (todo)");
    println!("- GitHub Issues: derive from git remotes and token (todo)");
    Ok(())
}
