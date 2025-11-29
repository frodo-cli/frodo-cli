mod cli;
mod config;
mod storage;
mod tui;

use clap::Parser;
use color_eyre::Result;
use frodo_core::storage::SecureStore;
use frodo_storage::secure_file_store::EncryptedFileStore;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Entry point wiring the CLI to the (placeholder) TUI.
#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    init_tracing();

    let cli = cli::Cli::parse();
    let config = config::load()?;
    match cli.command.unwrap_or(cli::Command::Tui) {
        cli::Command::Tui => tui::launch()?,
        cli::Command::Version => print_version(),
        cli::Command::Health => run_health_check(&config).await?,
    }

    Ok(())
}

fn init_tracing() {
    // Respect user-provided filters, default to info to avoid noisy stdout.
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let fmt_layer = tracing_subscriber::fmt::layer().with_target(false);
    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();
}

fn print_version() {
    println!("frodo-cli {}", env!("CARGO_PKG_VERSION"));
}

/// Runs a quick health check of the encrypted storage path.
async fn run_health_check(config: &config::Config) -> Result<()> {
    let store: EncryptedFileStore<_> = storage::store_from_config(config)?;
    run_store_health(&store).await?;
    println!("Storage: ok");
    Ok(())
}

async fn run_store_health<S: SecureStore>(store: &S) -> Result<()> {
    let probe_key = "health/probe";
    let payload = b"ok";
    store
        .put(probe_key, payload)
        .await
        .map_err(|e| color_eyre::eyre::eyre!(e.to_string()))?;
    let round_trip = store
        .get(probe_key)
        .await
        .map_err(|e| color_eyre::eyre::eyre!(e.to_string()))?;
    store
        .delete(probe_key)
        .await
        .map_err(|e| color_eyre::eyre::eyre!(e.to_string()))?;

    if round_trip != payload {
        color_eyre::eyre::bail!("storage round-trip failed");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage;

    #[tokio::test]
    async fn health_check_with_test_store_succeeds() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = storage::test_store(dir.path());
        run_store_health(&store)
            .await
            .expect("health check should succeed");
    }
}
