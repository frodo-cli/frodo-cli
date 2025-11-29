mod cli;
mod tui;

use clap::Parser;
use color_eyre::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Entry point wiring the CLI to the (placeholder) TUI.
#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    init_tracing();

    let cli = cli::Cli::parse();
    match cli.command.unwrap_or(cli::Command::Tui) {
        cli::Command::Tui => tui::launch()?,
        cli::Command::Version => print_version(),
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
