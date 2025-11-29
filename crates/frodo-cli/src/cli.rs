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
    /// Ask the agent a question from the terminal.
    Ask {
        /// The question/prompt to send to the agent.
        #[arg(required = true)]
        prompt: Vec<String>,
    },
    /// Sync tasks with remote providers (Jira/GitHub) — currently a stub.
    Sync,
    /// Manage tasks.
    #[command(subcommand)]
    Task(TaskCommand),
}

#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum ConfigCommand {
    /// Create a default config file if one does not exist.
    Init,
}

#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum TaskCommand {
    /// List tasks.
    List,
    /// Add a new task.
    Add {
        /// Title for the task.
        title: String,
        /// Optional description.
        #[arg(short, long)]
        description: Option<String>,
        /// Tags for grouping (repeat flag).
        #[arg(short, long)]
        tag: Vec<String>,
    },
    /// Mark a task as done.
    Done {
        /// Task id (UUID).
        id: String,
    },
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

    #[test]
    fn parses_ask_subcommand() {
        let cli = Cli::try_parse_from(["frodo", "ask", "hello", "world"]).expect("parse ok");
        assert_eq!(
            cli.command,
            Some(Command::Ask {
                prompt: vec!["hello".into(), "world".into()]
            })
        );
    }

    #[test]
    fn parses_task_add() {
        let cli = Cli::try_parse_from([
            "frodo",
            "task",
            "add",
            "title",
            "--description",
            "desc",
            "--tag",
            "one",
            "--tag",
            "two",
        ])
        .expect("parse ok");
        assert_eq!(
            cli.command,
            Some(Command::Task(TaskCommand::Add {
                title: "title".into(),
                description: Some("desc".into()),
                tag: vec!["one".into(), "two".into()],
            }))
        );
    }

    #[test]
    fn parses_task_done() {
        let cli = Cli::try_parse_from(["frodo", "task", "done", "123"]).expect("parse ok");
        assert_eq!(
            cli.command,
            Some(Command::Task(TaskCommand::Done { id: "123".into() }))
        );
    }

    #[test]
    fn parses_sync_subcommand() {
        let cli = Cli::try_parse_from(["frodo", "sync"]).expect("parse ok");
        assert_eq!(cli.command, Some(Command::Sync));
    }
}
