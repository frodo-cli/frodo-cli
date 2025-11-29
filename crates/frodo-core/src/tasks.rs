use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Task status lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
}

impl Default for TaskStatus {
    fn default() -> Self {
        TaskStatus::Todo
    }
}

/// Task entity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub status: TaskStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Task {
    pub fn new(title: String, description: Option<String>, tags: Vec<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title,
            description,
            tags,
            status: TaskStatus::Todo,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Simple repository contract for task persistence.
#[async_trait]
pub trait TaskRepository: Send + Sync {
    async fn list(&self) -> anyhow::Result<Vec<Task>>;
    async fn create(
        &self,
        title: String,
        description: Option<String>,
        tags: Vec<String>,
    ) -> anyhow::Result<Task>;
    async fn set_status(&self, id: Uuid, status: TaskStatus) -> anyhow::Result<Task>;
}
