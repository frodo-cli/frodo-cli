mod cli;
mod config;
mod storage;
mod sync;
mod tasks;
mod tui;

use crate::cli::ConfigCommand;
use clap::Parser;
use color_eyre::Result;
use frodo_agent::openai::{OpenAiAgent, OpenAiSettings};
use frodo_core::{
    agent::{Agent, AgentContext, AgentRequest, AgentResponse, EchoAgent},
    storage::SecureStore,
    tasks::{Task, TaskRepository},
};
use frodo_storage::secure_file_store::EncryptedFileStore;
use frodo_task::SecureStoreTaskRepo;
use std::collections::BTreeMap;
use std::sync::Arc;
use tracing::warn;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Entry point wiring the CLI to the (placeholder) TUI.
#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    init_tracing();

    let cli = cli::Cli::parse();
    let config = config::load()?;
    match cli.command.unwrap_or(cli::Command::Tui) {
        cli::Command::Tui => {
            let tasks = load_tasks(&config).await?;
            tui::launch(&tasks)?
        }
        cli::Command::Version => print_version(),
        cli::Command::Health => run_health_check(&config).await?,
        cli::Command::Config(ConfigCommand::Init) => init_config(&config)?,
        cli::Command::Ask { prompt } => run_ask(prompt, &config).await?,
        cli::Command::Task(cmd) => tasks::handle(cmd, &config).await?,
        cli::Command::Sync => sync::run(&config).await?,
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

fn init_config(config: &config::Config) -> Result<()> {
    let path = config::write_default_if_missing(config)?;
    println!("Config initialized at {}", path.display());
    Ok(())
}

async fn run_ask(prompt: Vec<String>, config: &config::Config) -> Result<()> {
    let prompt_text = prompt.join(" ");
    let (agent_name, agent) = build_agent(config)?;
    let response = ask_with_agent(agent.as_ref(), prompt_text).await?;
    println!("[{agent_name}] {}", response.message.content);
    if let Some(summary) = response.summary {
        println!("\nSummary: {summary}");
    }
    Ok(())
}

async fn ask_with_agent(
    agent: &(dyn Agent + Send + Sync),
    prompt: String,
) -> Result<AgentResponse> {
    let request = AgentRequest {
        prompt,
        conversation_id: None,
        context: AgentContext {
            workspace: None,
            hints: BTreeMap::new(),
        },
    };
    agent
        .ask(request)
        .await
        .map_err(|e| color_eyre::eyre::eyre!(e.to_string()))
}

fn build_agent(config: &config::Config) -> Result<(String, Arc<dyn Agent + Send + Sync>)> {
    if let Some(settings) = resolve_openai_settings(config) {
        match OpenAiAgent::new(settings.clone()) {
            Ok(agent) => return Ok((agent.name().to_string(), Arc::new(agent))),
            Err(err) => warn!("failed to init OpenAI agent, falling back to echo: {err}"),
        }
    }

    let agent = EchoAgent;
    Ok((agent.name().to_string(), Arc::new(agent)))
}

fn resolve_openai_settings(config: &config::Config) -> Option<OpenAiSettings> {
    let key = config
        .openai
        .as_ref()
        .and_then(|c| c.api_key.clone())
        .or_else(|| std::env::var("FRODO_OPENAI_API_KEY").ok())
        .or_else(|| std::env::var("OPENAI_API_KEY").ok());

    let model = config
        .openai
        .as_ref()
        .and_then(|c| c.model.clone())
        .unwrap_or_else(|| "gpt-4o-mini".to_string());

    let api_base = config.openai.as_ref().and_then(|c| c.endpoint.clone());

    key.map(|api_key| OpenAiSettings {
        api_key,
        model,
        api_base,
    })
}

async fn load_tasks(config: &config::Config) -> Result<Vec<Task>> {
    let store = storage::store_from_config(config)?;
    let repo: SecureStoreTaskRepo<_> = SecureStoreTaskRepo::new(store);
    repo.list()
        .await
        .map_err(|e| color_eyre::eyre::eyre!(e.to_string()))
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

    #[tokio::test]
    async fn ask_with_echo_agent_returns_echoed_content() {
        let agent = EchoAgent;
        let response = ask_with_agent(&agent, "hello world".into())
            .await
            .expect("ask should succeed");
        assert_eq!(response.message.content, "Echo: hello world");
        assert_eq!(response.summary.as_deref(), Some("echo stub"));
    }
}
