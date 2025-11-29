use color_eyre::Result;
use frodo_core::tasks::{TaskRepository, TaskStatus};
use frodo_storage::secure_file_store::EncryptedFileStore;
use frodo_task::SecureStoreTaskRepo;
use uuid::Uuid;

use crate::{cli::TaskCommand, config, storage};

/// Execute a task subcommand using the encrypted store.
pub async fn handle(cmd: TaskCommand, config: &config::Config) -> Result<()> {
    let repo: SecureStoreTaskRepo<EncryptedFileStore<_>> =
        SecureStoreTaskRepo::new(storage::store_from_config(config)?);

    match cmd {
        TaskCommand::List => {
            let tasks = repo
                .list()
                .await
                .map_err(|e| color_eyre::eyre::eyre!(e.to_string()))?;
            if tasks.is_empty() {
                println!("No tasks yet. Add one with `frodo task add <title>`.");
                return Ok(());
            }
            for task in tasks {
                println!(
                    "{} [{}] {}",
                    task.id,
                    status_label(&task.status),
                    task.title
                );
                if let Some(desc) = task.description {
                    println!("    {desc}");
                }
                if !task.tags.is_empty() {
                    println!("    tags: {}", task.tags.join(", "));
                }
            }
        }
        TaskCommand::Add {
            title,
            description,
            tag,
        } => {
            let task = repo
                .create(title, description, tag)
                .await
                .map_err(|e| color_eyre::eyre::eyre!(e.to_string()))?;
            println!("Created task {}: {}", task.id, task.title);
        }
        TaskCommand::Done { id } => {
            let uuid = Uuid::parse_str(&id).map_err(|e| color_eyre::eyre::eyre!(e.to_string()))?;
            let task = repo
                .set_status(uuid, TaskStatus::Done)
                .await
                .map_err(|e| color_eyre::eyre::eyre!(e.to_string()))?;
            println!("Marked done: {}", task.title);
        }
    }

    Ok(())
}

fn status_label(status: &TaskStatus) -> &'static str {
    match status {
        TaskStatus::Todo => "todo",
        TaskStatus::InProgress => "doing",
        TaskStatus::Done => "done",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use frodo_core::storage::InMemorySecureStore;
    use frodo_task::SecureStoreTaskRepo;

    #[tokio::test]
    async fn task_repo_round_trip() {
        let repo = SecureStoreTaskRepo::new(InMemorySecureStore::new());
        let created = repo
            .create("Example".into(), None, vec!["tag".into()])
            .await
            .expect("create");
        let listed = repo.list().await.expect("list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, created.id);
    }
}
