use std::collections::BTreeMap;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Incoming request for an agent invocation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentRequest {
    /// Free-form user prompt.
    pub prompt: String,
    /// Optional conversation identifier to thread messages.
    pub conversation_id: Option<String>,
    /// Structured context passed alongside the prompt.
    pub context: AgentContext,
}

/// Context fed to agents to ground answers.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentContext {
    /// Active workspace or repo identifier (e.g., `org/repo`).
    pub workspace: Option<String>,
    /// Arbitrary key/value hints (e.g., branch name, ticket id).
    pub hints: BTreeMap<String, String>,
}

/// Agent response payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentResponse {
    /// Main message the agent wants to show the user.
    pub message: AgentMessage,
    /// Optional concise summary (for notifications or list views).
    pub summary: Option<String>,
}

/// Agent message content.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentMessage {
    /// Human-readable text (markdown allowed).
    pub content: String,
}

/// Contract for any agent provider (OpenAI, local, stub).
#[async_trait]
pub trait Agent: Send + Sync {
    /// Short name used for logging and UI.
    fn name(&self) -> &'static str;

    /// Ask the agent to respond to a prompt with context.
    async fn ask(&self, request: AgentRequest) -> Result<AgentResponse>;
}

/// Simple agent that echoes the prompt and is useful for tests and offline smoke checks.
pub struct EchoAgent;

#[async_trait]
impl Agent for EchoAgent {
    fn name(&self) -> &'static str {
        "echo"
    }

    async fn ask(&self, request: AgentRequest) -> Result<AgentResponse> {
        Ok(AgentResponse {
            message: AgentMessage {
                content: format!("Echo: {}", request.prompt),
            },
            summary: Some("echo stub".to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn echo_agent_round_trips_prompt() {
        let agent = EchoAgent;
        let response = agent
            .ask(AgentRequest {
                prompt: "hello".into(),
                conversation_id: Some("conv-1".into()),
                context: AgentContext {
                    workspace: Some("org/repo".into()),
                    hints: BTreeMap::from([("branch".into(), "main".into())]),
                },
            })
            .await
            .expect("echo agent should succeed");

        assert_eq!(response.message.content, "Echo: hello");
        assert_eq!(response.summary.as_deref(), Some("echo stub"));
    }
}
