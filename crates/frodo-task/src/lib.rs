use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use frodo_core::{
    storage::{SecureStore, SecureStoreError},
    tasks::{Task, TaskRepository, TaskStatus},
};
use tracing::instrument;
use uuid::Uuid;

const TASKS_KEY: &str = "tasks";

/// Task repository backed by a `SecureStore` (encrypted at rest).
pub struct SecureStoreTaskRepo<S: SecureStore> {
    store: Arc<S>,
}

impl<S: SecureStore> SecureStoreTaskRepo<S> {
    pub fn new(store: S) -> Self {
        Self {
            store: Arc::new(store),
        }
    }

    async fn load(&self) -> Result<Vec<Task>> {
        match self.store.get(TASKS_KEY).await {
            Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
            Err(SecureStoreError::NotFound { .. }) => Ok(Vec::new()),
            Err(err) => Err(anyhow::anyhow!(err.to_string())),
        }
    }

    async fn save(&self, tasks: &[Task]) -> Result<()> {
        let bytes = serde_json::to_vec(tasks)?;
        self.store
            .put(TASKS_KEY, &bytes)
            .await
            .map_err(|e| anyhow::anyhow!(e.to_string()))
    }
}

#[async_trait]
impl<S: SecureStore> TaskRepository for SecureStoreTaskRepo<S> {
    #[instrument(skip(self))]
    async fn list(&self) -> Result<Vec<Task>> {
        self.load().await
    }

    #[instrument(skip(self, description, tags))]
    async fn create(
        &self,
        title: String,
        description: Option<String>,
        tags: Vec<String>,
    ) -> Result<Task> {
        let mut tasks = self.load().await?;
        let task = Task::new(title, description, tags);
        tasks.push(task.clone());
        self.save(&tasks).await?;
        Ok(task)
    }

    #[instrument(skip(self))]
    async fn set_status(&self, id: Uuid, status: TaskStatus) -> Result<Task> {
        let mut tasks = self.load().await?;
        let mut updated: Option<Task> = None;
        for task in &mut tasks {
            if task.id == id {
                task.status = status.clone();
                task.updated_at = chrono::Utc::now();
                updated = Some(task.clone());
                break;
            }
        }
        let updated = updated.ok_or_else(|| anyhow::anyhow!("task not found"))?;
        self.save(&tasks).await?;
        Ok(updated)
    }
}

#[cfg(test)]
mod tests {
    use frodo_core::{
        storage::InMemorySecureStore,
        tasks::{TaskRepository, TaskStatus},
    };

    use super::*;

    #[tokio::test]
    async fn creates_and_lists_tasks() {
        let repo = SecureStoreTaskRepo::new(InMemorySecureStore::new());
        let created = repo
            .create(
                "Write docs".into(),
                Some("MVP tasks".into()),
                vec!["docs".into()],
            )
            .await
            .expect("create");
        assert_eq!(created.status, TaskStatus::Todo);

        let tasks = repo.list().await.expect("list");
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "Write docs");
        assert_eq!(tasks[0].description.as_deref(), Some("MVP tasks"));
    }

    #[tokio::test]
    async fn updates_status() {
        let repo = SecureStoreTaskRepo::new(InMemorySecureStore::new());
        let created = repo
            .create("Ship".to_string(), None, vec![])
            .await
            .expect("create");

        let updated = repo
            .set_status(created.id, TaskStatus::Done)
            .await
            .expect("update");

        assert_eq!(updated.status, TaskStatus::Done);
        let tasks = repo.list().await.expect("list");
        assert_eq!(tasks[0].status, TaskStatus::Done);
    }
}
