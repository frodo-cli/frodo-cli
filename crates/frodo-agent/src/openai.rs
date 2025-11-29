use anyhow::{Context, Result};
use async_openai::{
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestSystemMessageContent, ChatCompletionRequestUserMessageArgs,
        ChatCompletionRequestUserMessageContent, CreateChatCompletionRequestArgs,
    },
    Client,
};
use async_trait::async_trait;
use frodo_core::agent::{Agent, AgentRequest, AgentResponse};
use serde::{Deserialize, Serialize};
use tracing::instrument;

/// Configuration for the OpenAI agent.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct OpenAiSettings {
    pub api_key: String,
    pub model: String,
    pub api_base: Option<String>,
}

impl OpenAiSettings {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            model: "gpt-4o-mini".to_string(),
            api_base: None,
        }
    }
}

/// OpenAI-backed agent using chat completions.
pub struct OpenAiAgent {
    client: Client<OpenAIConfig>,
    settings: OpenAiSettings,
}

impl OpenAiAgent {
    pub fn new(settings: OpenAiSettings) -> Result<Self> {
        let mut config = OpenAIConfig::new().with_api_key(&settings.api_key);
        if let Some(base) = &settings.api_base {
            config = config.with_api_base(base);
        }
        let client = Client::with_config(config);
        Ok(Self { client, settings })
    }

    fn system_prompt(&self) -> String {
        "You are Frodo, a concise, friendly developer teammate. Give short, actionable answers."
            .to_string()
    }
}

#[async_trait]
impl Agent for OpenAiAgent {
    fn name(&self) -> &'static str {
        "openai"
    }

    #[instrument(skip_all, fields(agent = "openai"))]
    async fn ask(&self, request: AgentRequest) -> Result<AgentResponse> {
        let system = ChatCompletionRequestMessage::System(
            ChatCompletionRequestSystemMessageArgs::default()
                .content(ChatCompletionRequestSystemMessageContent::Text(
                    self.system_prompt(),
                ))
                .build()
                .context("building system message")?,
        );

        let user = ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessageArgs::default()
                .content(ChatCompletionRequestUserMessageContent::Text(
                    request.prompt,
                ))
                .build()
                .context("building user message")?,
        );

        let req = CreateChatCompletionRequestArgs::default()
            .model(self.settings.model.clone())
            .messages(vec![system, user])
            .build()
            .context("building chat completion request")?;

        let resp = self
            .client
            .chat()
            .create(req)
            .await
            .context("openai chat completion failed")?;

        let choice = resp
            .choices
            .into_iter()
            .next()
            .context("openai returned no choices")?;
        let content = choice
            .message
            .content
            .unwrap_or_default()
            .trim()
            .to_string();

        Ok(AgentResponse {
            message: frodo_core::agent::AgentMessage { content },
            summary: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_default_model_and_base() {
        let settings = OpenAiSettings::new("key".into());
        assert_eq!(settings.model, "gpt-4o-mini");
        assert_eq!(settings.api_base, None);
    }
}
