use crate::changeset::RefinementChangeset;
use crate::error::RefinementError;
use crate::prompts::{generate_system_prompt, generate_user_prompt, RefinementMode};
use crate::provider::RefinementProvider;
use prompt_core::PromptDocument;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

/// Provider that connects to an OpenAI-compatible `/chat/completions` HTTP endpoint.
#[derive(Debug, Clone)]
pub struct OpenAiCompatibleProvider {
    pub base_url: String,
    pub model: String,
    pub api_key_env_var: String,
    pub timeout_seconds: u64,
}

impl OpenAiCompatibleProvider {
    pub fn new(
        base_url: impl Into<String>,
        model: impl Into<String>,
        api_key_env_var: impl Into<String>,
        timeout_seconds: u64,
    ) -> Self {
        Self {
            base_url: base_url.into(),
            model: model.into(),
            api_key_env_var: api_key_env_var.into(),
            timeout_seconds: if timeout_seconds == 0 { 60 } else { timeout_seconds },
        }
    }

    fn resolve_api_key(&self) -> Result<String, RefinementError> {
        std::env::var(&self.api_key_env_var)
            .map_err(|_| RefinementError::MissingApiKey(self.api_key_env_var.clone()))
    }
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    format_type: String,
}

#[derive(Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    response_format: ResponseFormat,
}

#[derive(Deserialize)]
struct ChatCompletionChoice {
    message: ChatResponseMessage,
}

#[derive(Deserialize)]
struct ChatResponseMessage {
    content: String,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatCompletionChoice>,
}

impl RefinementProvider for OpenAiCompatibleProvider {
    fn refine<'a>(
        &'a self,
        doc: &'a PromptDocument,
        mode: RefinementMode,
    ) -> Pin<Box<dyn Future<Output = Result<RefinementChangeset, RefinementError>> + Send + 'a>> {
        Box::pin(async move {
            let api_key = self.resolve_api_key()?;
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(self.timeout_seconds))
                .build()?;

            let url = if self.base_url.ends_with("/chat/completions") {
                self.base_url.clone()
            } else if self.base_url.ends_with('/') {
                format!("{}chat/completions", self.base_url)
            } else {
                format!("{}/chat/completions", self.base_url)
            };

            let system_prompt = generate_system_prompt(mode);
            let user_prompt = generate_user_prompt(doc);

            let request_payload = ChatCompletionRequest {
                model: self.model.clone(),
                messages: vec![
                    ChatMessage {
                        role: "system".into(),
                        content: system_prompt,
                    },
                    ChatMessage {
                        role: "user".into(),
                        content: user_prompt,
                    },
                ],
                temperature: 0.2,
                response_format: ResponseFormat {
                    format_type: "json_object".into(),
                },
            };

            let response = client
                .post(&url)
                .bearer_auth(api_key)
                .json(&request_payload)
                .send()
                .await?;

            let status = response.status();
            if !status.is_success() {
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Failed to read response body".into());
                return Err(RefinementError::ProviderError(format!(
                    "API returned status {status}: {error_text}"
                )));
            }

            let parsed_body: ChatCompletionResponse = response.json().await?;
            let content = parsed_body
                .choices
                .first()
                .map(|c| c.message.content.as_str())
                .ok_or_else(|| {
                    RefinementError::ProviderError("No response choices returned by model".into())
                })?;

            let changeset: RefinementChangeset = serde_json::from_str(content)?;
            changeset.validate_against_document(doc)?;

            Ok(changeset)
        })
    }
}
