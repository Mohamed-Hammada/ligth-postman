//! OpenAI Chat Completions API provider — also backs the `Custom` provider kind (LP-0822).

use reqwest::Client;
use serde::Deserialize;

use crate::error::AppError;

use super::{
    parse_and_validate, parse_sample_response, parse_tests_and_docs, AiProvider,
    GeneratedApiDefinition, GeneratedSampleResponse, GeneratedTestsAndDocs, SYSTEM_PROMPT_API,
    SYSTEM_PROMPT_SAMPLE_RESPONSE, SYSTEM_PROMPT_TESTS_DOCS,
};

/// OpenAI's own default base — the same base a "Custom" (OpenAI-compatible) provider would set
/// explicitly, since Groq/Together/OpenRouter/Ollama/LM Studio/etc. all speak this same
/// `{base_url}/chat/completions` shape and differ only in which base_url and key you point at.
pub(crate) const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";
pub(crate) const DEFAULT_MODEL: &str = "gpt-4o";

/// Talks to any OpenAI Chat Completions-shaped endpoint: the real OpenAI API when constructed
/// via [`Self::openai`], or any OpenAI-compatible one (Groq, Together, OpenRouter, Ollama,
/// LM Studio, Azure OpenAI, self-hosted vLLM, ...) via [`Self::custom`] with a user-supplied
/// base_url. Same wire format either way — `{base_url}/chat/completions`, `Authorization:
/// Bearer <key>`, `{"model","messages":[{role,content}]}` in, `choices[0].message.content` out.
#[derive(Clone)]
pub struct OpenAiCompatibleProvider {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
}

impl OpenAiCompatibleProvider {
    pub(super) fn openai(client: Client, api_key: String, base_url: Option<String>, model: String) -> Self {
        Self {
            client,
            api_key,
            base_url: base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_string()),
            model,
        }
    }

    /// `base_url` is required here (unlike [`Self::openai`]) — there is no sane default for an
    /// arbitrary self-hosted or third-party endpoint.
    pub(super) fn custom(client: Client, api_key: String, base_url: String, model: String) -> Self {
        Self { client, api_key, base_url, model }
    }

    #[cfg(test)]
    pub fn with_base_url(client: Client, api_key: &str, base_url: &str) -> Self {
        Self {
            client,
            api_key: api_key.to_string(),
            base_url: base_url.to_string(),
            model: DEFAULT_MODEL.to_string(),
        }
    }

    async fn send_openai_message(&self, system_prompt: &str, user_message: &str) -> Result<String, AppError> {
        let request_body = serde_json::json!({
            "model": self.model,
            "max_tokens": 2048,
            "messages": [
                { "role": "system", "content": system_prompt },
                { "role": "user", "content": user_message },
            ],
        });

        let endpoint = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let response = self
            .client
            .post(&endpoint)
            .header("authorization", format!("Bearer {}", self.api_key))
            .header("content-type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|err| AppError::Network(format!("AI request failed: {err}")))?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(AppError::Ai(format!("AI provider returned {status}: {text}")));
        }

        let parsed: OpenAiChatResponse = response
            .json()
            .await
            .map_err(|err| AppError::Ai(format!("failed to parse AI provider response: {err}")))?;

        parsed
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or_else(|| AppError::Ai("AI provider response contained no choices".into()))
    }
}

impl AiProvider for OpenAiCompatibleProvider {
    async fn generate_api(&self, prompt: &str) -> Result<GeneratedApiDefinition, AppError> {
        let trimmed = prompt.trim();
        if trimmed.is_empty() {
            return Err(AppError::Validation("AI prompt must not be empty".into()));
        }
        let raw_text = self.send_openai_message(SYSTEM_PROMPT_API, trimmed).await?;
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
        let raw_text = self.send_openai_message(SYSTEM_PROMPT_SAMPLE_RESPONSE, &user_prompt).await?;
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
        let raw_text = self.send_openai_message(SYSTEM_PROMPT_TESTS_DOCS, &user_prompt).await?;
        parse_tests_and_docs(&raw_text)
    }
}

#[derive(Deserialize)]
struct OpenAiChatResponse {
    #[serde(default)]
    choices: Vec<OpenAiChatChoice>,
}

#[derive(Deserialize)]
struct OpenAiChatChoice {
    message: OpenAiChatMessage,
}

#[derive(Deserialize)]
struct OpenAiChatMessage {
    #[serde(default)]
    content: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::test_support::spawn_mock_server_with_status;

    // Same three behaviors as the Anthropic provider, against the real choices[0].message.content
    // wire shape — proves "custom" providers work through the exact same code path as the real
    // OpenAI API.

    #[tokio::test]
    async fn openai_generate_api_end_to_end_against_a_real_http_response() {
        let openai_style_body = serde_json::json!({
            "id": "chatcmpl-test",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": r#"{"name":"List orders","method":"GET","url":"https://api.example.com/orders","headers":[],"query_params":[],"body":null,"description":"Fetch all orders"}"#
                },
                "finish_reason": "stop"
            }]
        })
        .to_string();

        let port = spawn_mock_server_with_status("200 OK", &openai_style_body);
        let provider = OpenAiCompatibleProvider::with_base_url(
            Client::new(),
            "test-key",
            &format!("http://127.0.0.1:{port}"),
        );

        let def = provider.generate_api("get all orders").await.unwrap();
        assert_eq!(def.name, "List orders");
        assert_eq!(def.method, "GET");
    }

    #[tokio::test]
    async fn openai_surfaces_non_success_status_as_ai_error() {
        let port = spawn_mock_server_with_status("401 Unauthorized", "{\"error\":\"bad key\"}");
        let provider = OpenAiCompatibleProvider::with_base_url(
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
}
