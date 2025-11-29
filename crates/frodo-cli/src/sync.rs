use color_eyre::Result;
use frodo_core::tasks::TaskRepository;
use frodo_storage::secure_file_store::EncryptedFileStore;
use frodo_sync::{GitHubConfig, GitHubSync, JiraConfig, JiraSync, NoopSync, TaskSync};
use frodo_task::SecureStoreTaskRepo;
use tracing::info;

use crate::config;
use crate::storage;

/// Placeholder sync handler. Uses a no-op sync provider for now.
pub async fn run(cfg: &config::Config, apply: bool) -> Result<()> {
    let provider = select_provider(cfg);
    info!(
        "sync invoked (provider: {}, apply={})",
        provider.name(),
        apply
    );
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
    let store: EncryptedFileStore<_> = storage::store_from_config(cfg)?;
    let repo: SecureStoreTaskRepo<_> = SecureStoreTaskRepo::new(store);
    let local = repo
        .list()
        .await
        .map_err(|e| color_eyre::eyre::eyre!(e.to_string()))?;
    println!("Local tasks: {}", local.len());
    if apply {
        provider
            .push(&[])
            .await
            .map_err(|e| color_eyre::eyre::eyre!(e.to_string()))?;
        println!("Applied push (stub).");
    } else {
        println!("Dry run: not pushing changes.");
    }
    Ok(())
}

fn select_provider(cfg: &config::Config) -> Box<dyn TaskSync> {
    if let Some(gh) = &cfg.github {
        let gh_cfg = GitHubConfig {
            owner: gh.owner.clone(),
            repo: gh.repo.clone(),
            token: gh.token.clone(),
        };
        return Box::new(GitHubSync::new(gh_cfg));
    }
    if let Some(jira) = &cfg.jira {
        let jira_cfg = JiraConfig {
            site: jira.site.clone(),
            project_key: jira.project_key.clone(),
            api_token: jira.api_token.clone(),
            email: jira.email.clone(),
        };
        return Box::new(JiraSync::new(jira_cfg));
    }
    Box::new(NoopSync)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_github_when_configured() {
        let cfg = config::Config {
            data_dir: None,
            openai: None,
            jira: None,
            github: Some(frodo_sync::GitHubConfig {
                owner: "o".into(),
                repo: "r".into(),
                token: "t".into(),
            }),
        };
        let provider = select_provider(&cfg);
        assert_eq!(provider.name(), "github");
    }

    #[test]
    fn selects_jira_when_configured() {
        let cfg = config::Config {
            data_dir: None,
            openai: None,
            jira: Some(frodo_sync::JiraConfig {
                site: "s".into(),
                project_key: "P".into(),
                api_token: "t".into(),
                email: "e".into(),
            }),
            github: None,
        };
        let provider = select_provider(&cfg);
        assert_eq!(provider.name(), "jira");
    }

    #[test]
    fn defaults_to_noop() {
        let provider = select_provider(&config::Config::default());
        assert_eq!(provider.name(), "noop");
    }
}
