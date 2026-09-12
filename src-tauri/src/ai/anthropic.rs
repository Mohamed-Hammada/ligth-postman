//! Anthropic Messages API provider (LP-0802).

use reqwest::Client;
use serde::Deserialize;

use crate::error::AppError;

use super::{
    parse_and_validate, parse_sample_response, parse_tests_and_docs, AiProvider,
    GeneratedApiDefinition, GeneratedSampleResponse, GeneratedTestsAndDocs, SYSTEM_PROMPT_API,
    SYSTEM_PROMPT_SAMPLE_RESPONSE, SYSTEM_PROMPT_TESTS_DOCS,
};

pub(crate) const DEFAULT_API_URL: &str = "https://api.anthropic.com/v1/messages";
pub(crate) const DEFAULT_MODEL: &str = "claude-sonnet-5";
const ANTHROPIC_VERSION: &str = "2023-06-01";

#[derive(Clone)]
pub struct ClaudeProvider {
    client: Client,
    api_key: String,
    api_url: String,
    model: String,
}

impl ClaudeProvider {
    /// Used by [`super::build_provider`] — kept as a named constructor rather than a public
    /// struct literal so the fields above can stay private to this file.
    pub(super) fn from_parts(client: Client, api_key: String, api_url: String, model: String) -> Self {
        Self { client, api_key, api_url, model }
    }

    /// Returns `None` when no key is configured in DB or environment.
    pub fn from_env(client: Client) -> Option<Self> {
        std::env::var("ANTHROPIC_API_KEY")
            .ok()
            .filter(|key| !key.trim().is_empty())
            .map(|api_key| Self {
                client,
                api_key,
                api_url: DEFAULT_API_URL.to_string(),
                model: DEFAULT_MODEL.to_string(),
            })
    }

    #[cfg(test)]
    pub fn with_base_url(client: Client, api_key: &str, api_url: &str) -> Self {
        Self {
            client,
            api_key: api_key.to_string(),
            api_url: api_url.to_string(),
            model: "claude-opus-5".to_string(),
        }
    }

    async fn send_anthropic_message(&self, system_prompt: &str, user_message: &str) -> Result<String, AppError> {
        let request_body = serde_json::json!({
            "model": self.model,
            "max_tokens": 2048,
            "system": system_prompt,
            "messages": [{ "role": "user", "content": user_message }],
        });

        let response = self
            .client
            .post(&self.api_url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|err| AppError::Network(format!("AI request failed: {err}")))?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(AppError::Ai(format!("Claude API returned {status}: {text}")));
        }

        let parsed: AnthropicMessageResponse = response
            .json()
            .await
            .map_err(|err| AppError::Ai(format!("failed to parse Claude API response: {err}")))?;

        parsed
            .content
            .into_iter()
            .find_map(|block| (block.block_type == "text").then_some(block.text))
            .ok_or_else(|| AppError::Ai("Claude response contained no text block".into()))
    }
}

impl AiProvider for ClaudeProvider {
    async fn generate_api(&self, prompt: &str) -> Result<GeneratedApiDefinition, AppError> {
        let trimmed = prompt.trim();
        if trimmed.is_empty() {
            return Err(AppError::Validation("AI prompt must not be empty".into()));
        }

        let raw_text = self.send_anthropic_message(SYSTEM_PROMPT_API, trimmed).await?;
        parse_and_validate(&raw_text)
    }

    async fn generate_sample_response(
        &self,
        method: &str,
        url: &str,
        body: Option<&str>,
        description: Option<&str>,
    ) -> Result<GeneratedSampleResponse, AppError> {
        let user_prompt = format!(
            "Generate a sample mock response for: Method: {method}, URL: {url}\nDescription: {}\nRequest Body: {}",
            description.unwrap_or("none"),
            body.unwrap_or("none")
        );

        let raw_text = self.send_anthropic_message(SYSTEM_PROMPT_SAMPLE_RESPONSE, &user_prompt).await?;
        parse_sample_response(&raw_text)
    }

    async fn generate_tests_and_docs(
        &self,
        method: &str,
        url: &str,
        body: Option<&str>,
    ) -> Result<GeneratedTestsAndDocs, AppError> {
        let user_prompt = format!(
            "Generate test assertions and markdown documentation for:\nMethod: {method}\nURL: {url}\nBody: {}",
            body.unwrap_or("none")
        );

        let raw_text = self.send_anthropic_message(SYSTEM_PROMPT_TESTS_DOCS, &user_prompt).await?;
        parse_tests_and_docs(&raw_text)
    }
}

#[derive(Deserialize)]
struct AnthropicMessageResponse {
    content: Vec<AnthropicContentBlock>,
}

#[derive(Deserialize)]
struct AnthropicContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    #[serde(default)]
    text: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::test_support::spawn_mock_server_with_status;

    fn spawn_mock_anthropic_server(response_body: &str) -> u16 {
        spawn_mock_server_with_status("200 OK", response_body)
    }

    #[tokio::test]
    async fn generate_api_end_to_end_against_a_real_http_response() {
        let anthropic_style_body = serde_json::json!({
            "id": "msg_test",
            "type": "message",
            "role": "assistant",
            "content": [{
                "type": "text",
                "text": r#"{"name":"List orders","method":"GET","url":"https://api.example.com/orders","headers":[],"query_params":[],"body":null,"description":"Fetch all orders"}"#
            }]
        })
        .to_string();

        let port = spawn_mock_anthropic_server(&anthropic_style_body);
        let provider = ClaudeProvider::with_base_url(
            Client::new(),
            "test-key",
            &format!("http://127.0.0.1:{port}"),
        );

        let def = provider.generate_api("get all orders").await.unwrap();
        assert_eq!(def.name, "List orders");
        assert_eq!(def.method, "GET");
        assert_eq!(def.url, "https://api.example.com/orders");
    }

    #[tokio::test]
    async fn generate_api_surfaces_non_success_status_as_ai_error() {
        let port = spawn_mock_server_with_status("401 Unauthorized", "{\"error\":\"bad key\"}");

        let provider = ClaudeProvider::with_base_url(
            Client::new(),
            "bad-key",
            &format!("http://127.0.0.1:{port}"),
        );

        let err = provider.generate_api("hello").await.unwrap_err();
        match err {
            AppError::Ai(msg) => assert!(msg.contains("401")),
            other => panic!("expected AppError::Ai, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn empty_prompt_is_rejected_without_a_network_call() {
        let provider = ClaudeProvider::with_base_url(
            Client::new(),
            "irrelevant",
            "http://127.0.0.1:1",
        );
        let err = provider.generate_api("   ").await.unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }
}
