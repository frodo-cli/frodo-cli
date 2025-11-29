use clap::{Parser, Subcommand};

/// CLI surface definition. Kept tiny for now; will expand with task/sync/chat commands.
#[derive(Parser, Debug)]
#[command(
    name = "frodo",
    about = "Local-first, team-friendly developer companion",
    version,
    propagate_version = true
)]
pub struct Cli {
    /// Optional subcommand; defaults to launching the TUI when absent.
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Launch the interactive TUI (press q or Esc to exit).
    Tui,
    /// Print version and exit.
    Version,
    /// Run a health check against core subsystems (storage, config).
    Health,
    /// Manage CLI configuration.
    #[command(subcommand)]
    Config(ConfigCommand),
}

#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum ConfigCommand {
    /// Create a default config file if one does not exist.
    Init,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tui_subcommand() {
        let cli = Cli::try_parse_from(["frodo", "tui"]).expect("parse should succeed");
        assert_eq!(cli.command, Some(Command::Tui));
    }

    #[test]
    fn defaults_to_tui_when_missing_subcommand() {
        let cli = Cli::try_parse_from(["frodo"]).expect("parse should succeed");
        assert_eq!(cli.command, None);
    }

    #[test]
    fn parses_health_subcommand() {
        let cli = Cli::try_parse_from(["frodo", "health"]).expect("parse should succeed");
        assert_eq!(cli.command, Some(Command::Health));
    }

    #[test]
    fn parses_config_init_subcommand() {
        let cli = Cli::try_parse_from(["frodo", "config", "init"]).expect("parse should succeed");
        assert_eq!(cli.command, Some(Command::Config(ConfigCommand::Init)));
    }
}
