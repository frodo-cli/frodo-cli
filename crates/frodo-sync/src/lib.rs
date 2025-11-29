use anyhow::Result;
use async_trait::async_trait;
use frodo_core::tasks::Task;
use serde::{Deserialize, Serialize};
use tracing::instrument;

/// High-level sync contract for pulling/pushing tasks to remote providers.
#[async_trait]
pub trait TaskSync: Send + Sync {
    /// Human-readable provider name (e.g., "jira", "github").
    fn name(&self) -> &'static str;

    /// Pull tasks from remote and return merged view (caller handles conflict policy).
    async fn pull(&self) -> Result<Vec<Task>>;

    /// Push local tasks upstream (caller can scope which tasks).
    async fn push(&self, tasks: &[Task]) -> Result<()>;
}

/// No-op sync provider used as a placeholder.
pub struct NoopSync;

#[async_trait]
impl TaskSync for NoopSync {
    fn name(&self) -> &'static str {
        "noop"
    }

    #[instrument(skip_all)]
    async fn pull(&self) -> Result<Vec<Task>> {
        Ok(Vec::new())
    }

    #[instrument(skip_all)]
    async fn push(&self, _tasks: &[Task]) -> Result<()> {
        Ok(())
    }
}

/// Jira configuration placeholder.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct JiraConfig {
    pub site: String,
    pub project_key: String,
    pub api_token: String,
    pub email: String,
}

/// GitHub configuration placeholder.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct GitHubConfig {
    pub owner: String,
    pub repo: String,
    pub token: String,
}

pub struct JiraSync {
    cfg: JiraConfig,
}

impl JiraSync {
    pub fn new(cfg: JiraConfig) -> Self {
        Self { cfg }
    }
}

#[async_trait]
impl TaskSync for JiraSync {
    fn name(&self) -> &'static str {
        "jira"
    }

    #[instrument(skip_all, fields(site = %self.cfg.site, project = %self.cfg.project_key))]
    async fn pull(&self) -> Result<Vec<Task>> {
        // Placeholder: integrate Jira REST API here.
        Ok(Vec::new())
    }

    #[instrument(skip_all, fields(site = %self.cfg.site, project = %self.cfg.project_key))]
    async fn push(&self, _tasks: &[Task]) -> Result<()> {
        // Placeholder: integrate Jira REST API here.
        Ok(())
    }
}

pub struct GitHubSync {
    cfg: GitHubConfig,
}

impl GitHubSync {
    pub fn new(cfg: GitHubConfig) -> Self {
        Self { cfg }
    }
}

#[async_trait]
impl TaskSync for GitHubSync {
    fn name(&self) -> &'static str {
        "github"
    }

    #[instrument(skip_all, fields(repo = %self.cfg.repo, owner = %self.cfg.owner))]
    async fn pull(&self) -> Result<Vec<Task>> {
        // Placeholder: integrate GitHub Issues API here.
        Ok(Vec::new())
    }

    #[instrument(skip_all, fields(repo = %self.cfg.repo, owner = %self.cfg.owner))]
    async fn push(&self, _tasks: &[Task]) -> Result<()> {
        // Placeholder: integrate GitHub Issues API here.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn noop_round_trips() {
        let sync = NoopSync;
        assert_eq!(sync.name(), "noop");
        assert!(sync.pull().await.unwrap().is_empty());
        sync.push(&[]).await.unwrap();
    }

    #[test]
    fn provider_names() {
        let jira = JiraSync::new(JiraConfig {
            site: "https://example".into(),
            project_key: "PRJ".into(),
            api_token: "t".into(),
            email: "e@example.com".into(),
        });
        assert_eq!(jira.name(), "jira");

        let gh = GitHubSync::new(GitHubConfig {
            owner: "o".into(),
            repo: "r".into(),
            token: "t".into(),
        });
        assert_eq!(gh.name(), "github");
    }
}
