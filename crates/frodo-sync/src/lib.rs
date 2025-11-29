use anyhow::Result;
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono::{DateTime, Utc};
use frodo_core::tasks::{Task, TaskStatus};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::instrument;
use uuid::Uuid;

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
    #[serde(default)]
    pub base_url: Option<String>,
}

/// GitHub configuration placeholder.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct GitHubConfig {
    pub owner: String,
    pub repo: String,
    pub token: String,
    #[serde(default)]
    pub api_base: Option<String>,
}

pub struct JiraSync {
    cfg: JiraConfig,
    client: reqwest::Client,
}

impl JiraSync {
    pub fn new(cfg: JiraConfig) -> Self {
        Self {
            cfg,
            client: reqwest::Client::new(),
        }
    }

    fn headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("frodo-cli"));
        let basic = BASE64.encode(format!("{}:{}", self.cfg.email, self.cfg.api_token));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Basic {}", basic))?,
        );
        Ok(headers)
    }

    fn base_url(&self) -> String {
        self.cfg
            .base_url
            .as_deref()
            .unwrap_or_else(|| self.cfg.site.as_str())
            .trim_end_matches('/')
            .to_string()
    }
}

#[async_trait]
impl TaskSync for JiraSync {
    fn name(&self) -> &'static str {
        "jira"
    }

    #[instrument(skip_all, fields(site = %self.cfg.site, project = %self.cfg.project_key))]
    async fn pull(&self) -> Result<Vec<Task>> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("frodo-cli"));
        let basic = BASE64.encode(format!("{}:{}", self.cfg.email, self.cfg.api_token));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Basic {}", basic))?,
        );

        let jql = format!("project={}", self.cfg.project_key);
        let url = format!(
            "{}/rest/api/3/search",
            self.cfg
                .base_url
                .as_deref()
                .unwrap_or_else(|| self.cfg.site.as_str())
                .trim_end_matches('/')
        );
        let resp: JiraSearchResponse = self
            .client
            .post(&url)
            .headers(headers)
            .json(&serde_json::json!({ "jql": jql, "fields": ["summary", "description", "status", "labels", "updated"] }))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(resp.issues.into_iter().map(task_from_jira).collect())
    }

    #[instrument(skip_all, fields(site = %self.cfg.site, project = %self.cfg.project_key))]
    async fn push(&self, _tasks: &[Task]) -> Result<()> {
        for task in _tasks {
            let headers = self.headers()?;
            let url = format!("{}/rest/api/3/issue", self.base_url());
            let body = json!({
                "fields": {
                    "project": { "key": self.cfg.project_key },
                    "summary": task.title,
                    "description": task.description.clone().unwrap_or_default(),
                    "issuetype": { "name": "Task" },
                    "labels": task.tags,
                }
            });
            self.client
                .post(&url)
                .headers(headers)
                .json(&body)
                .send()
                .await?
                .error_for_status()?;
        }
        Ok(())
    }
}

pub struct GitHubSync {
    cfg: GitHubConfig,
    client: reqwest::Client,
}

impl GitHubSync {
    pub fn new(cfg: GitHubConfig) -> Self {
        Self {
            cfg,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl TaskSync for GitHubSync {
    fn name(&self) -> &'static str {
        "github"
    }

    #[instrument(skip_all, fields(repo = %self.cfg.repo, owner = %self.cfg.owner))]
    async fn pull(&self) -> Result<Vec<Task>> {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("token {}", self.cfg.token))?,
        );
        headers.insert(USER_AGENT, HeaderValue::from_static("frodo-cli"));
        let base = self
            .cfg
            .api_base
            .as_deref()
            .unwrap_or("https://api.github.com");
        let url = format!(
            "{base}/repos/{}/{}/issues?state=all",
            self.cfg.owner, self.cfg.repo
        );
        let issues: Vec<GitHubIssue> = self
            .client
            .get(&url)
            .headers(headers)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(issues.into_iter().map(task_from_github).collect())
    }

    #[instrument(skip_all, fields(repo = %self.cfg.repo, owner = %self.cfg.owner))]
    async fn push(&self, _tasks: &[Task]) -> Result<()> {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("token {}", self.cfg.token))?,
        );
        headers.insert(USER_AGENT, HeaderValue::from_static("frodo-cli"));
        let base = self
            .cfg
            .api_base
            .as_deref()
            .unwrap_or("https://api.github.com");
        let url = format!("{base}/repos/{}/{}/issues", self.cfg.owner, self.cfg.repo);
        for task in _tasks {
            let body = json!({
                "title": task.title,
                "body": task.description.clone().unwrap_or_default(),
            });
            self.client
                .post(&url)
                .headers(headers.clone())
                .json(&body)
                .send()
                .await?
                .error_for_status()?;
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GitHubIssue {
    title: String,
    body: Option<String>,
    state: String,
    labels: Option<Vec<GitHubLabel>>,
    updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubLabel {
    name: String,
}

fn task_from_github(issue: GitHubIssue) -> Task {
    let updated = issue
        .updated_at
        .as_deref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(Utc::now);
    let status = match issue.state.as_str() {
        "closed" => TaskStatus::Done,
        _ => TaskStatus::Todo,
    };
    Task {
        id: Uuid::new_v4(),
        title: issue.title,
        description: issue.body,
        tags: issue
            .labels
            .unwrap_or_default()
            .into_iter()
            .map(|l| l.name)
            .collect(),
        status,
        created_at: updated,
        updated_at: updated,
    }
}

#[derive(Debug, Deserialize)]
struct JiraSearchResponse {
    issues: Vec<JiraIssue>,
}

#[derive(Debug, Deserialize)]
struct JiraIssue {
    fields: JiraFields,
}

#[derive(Debug, Deserialize)]
struct JiraFields {
    summary: String,
    description: Option<String>,
    status: JiraStatus,
    #[serde(default)]
    labels: Vec<String>,
    #[serde(default)]
    updated: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
struct JiraStatus {
    name: String,
}

fn task_from_jira(issue: JiraIssue) -> Task {
    let updated = issue.fields.updated.unwrap_or_else(Utc::now);
    let status = match issue.fields.status.name.to_lowercase().as_str() {
        "done" | "closed" | "resolved" => TaskStatus::Done,
        "in progress" => TaskStatus::InProgress,
        _ => TaskStatus::Todo,
    };
    Task {
        id: Uuid::new_v4(),
        title: issue.fields.summary,
        description: issue.fields.description,
        tags: issue.fields.labels,
        status,
        created_at: updated,
        updated_at: updated,
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
            base_url: None,
        });
        assert_eq!(jira.name(), "jira");

        let gh = GitHubSync::new(GitHubConfig {
            owner: "o".into(),
            repo: "r".into(),
            token: "t".into(),
            api_base: None,
        });
        assert_eq!(gh.name(), "github");
    }
}
