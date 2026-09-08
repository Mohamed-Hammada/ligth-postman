//! Google Gemini `generateContent` REST API provider (LP-0822).

use reqwest::Client;
use serde::Deserialize;

use crate::error::AppError;

use super::{
    parse_and_validate, parse_sample_response, parse_tests_and_docs, AiProvider,
    GeneratedApiDefinition, GeneratedSampleResponse, GeneratedTestsAndDocs, SYSTEM_PROMPT_API,
    SYSTEM_PROMPT_SAMPLE_RESPONSE, SYSTEM_PROMPT_TESTS_DOCS,
};

pub(crate) const DEFAULT_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta";
pub(crate) const DEFAULT_MODEL: &str = "gemini-1.5-pro";

#[derive(Clone)]
pub struct GoogleProvider {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
}

impl GoogleProvider {
    pub(super) fn new(client: Client, api_key: String, base_url: Option<String>, model: String) -> Self {
        Self {
            client,
            api_key,
            base_url: base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_string()),
            model,
        }
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

    async fn send_google_message(&self, system_prompt: &str, user_message: &str) -> Result<String, AppError> {
        let request_body = serde_json::json!({
            "system_instruction": { "parts": [{ "text": system_prompt }] },
            "contents": [{ "role": "user", "parts": [{ "text": user_message }] }],
            "generationConfig": { "maxOutputTokens": 2048 },
        });

        let endpoint = format!(
            "{}/models/{}:generateContent",
            self.base_url.trim_end_matches('/'),
            self.model
        );
        let response = self
            .client
            .post(&endpoint)
            // Header auth (not `?key=` in the URL) so the key never ends up in a logged URL —
            // same care this app already takes redacting secrets from console/network logs.
            .header("x-goog-api-key", &self.api_key)
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

        let parsed: GoogleGenerateContentResponse = response
            .json()
            .await
            .map_err(|err| AppError::Ai(format!("failed to parse AI provider response: {err}")))?;

        parsed
            .candidates
            .into_iter()
            .next()
            .and_then(|c| c.content.parts.into_iter().next())
            .map(|p| p.text)
            .ok_or_else(|| AppError::Ai("AI provider response contained no candidates".into()))
    }
}

impl AiProvider for GoogleProvider {
    async fn generate_api(&self, prompt: &str) -> Result<GeneratedApiDefinition, AppError> {
        let trimmed = prompt.trim();
        if trimmed.is_empty() {
            return Err(AppError::Validation("AI prompt must not be empty".into()));
        }
        let raw_text = self.send_google_message(SYSTEM_PROMPT_API, trimmed).await?;
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
        let raw_text = self.send_google_message(SYSTEM_PROMPT_SAMPLE_RESPONSE, &user_prompt).await?;
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
        let raw_text = self.send_google_message(SYSTEM_PROMPT_TESTS_DOCS, &user_prompt).await?;
        parse_tests_and_docs(&raw_text)
    }
}

#[derive(Deserialize)]
struct GoogleGenerateContentResponse {
    #[serde(default)]
    candidates: Vec<GoogleCandidate>,
}

#[derive(Deserialize)]
struct GoogleCandidate {
    content: GoogleContent,
}

#[derive(Deserialize)]
struct GoogleContent {
    #[serde(default)]
    parts: Vec<GooglePart>,
}

#[derive(Deserialize)]
struct GooglePart {
    #[serde(default)]
    text: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::test_support::spawn_mock_server_with_status;

    // Against the real candidates[0].content.parts[0].text wire shape.

    #[tokio::test]
    async fn google_generate_api_end_to_end_against_a_real_http_response() {
        let gemini_style_body = serde_json::json!({
            "candidates": [{
                "content": {
                    "role": "model",
                    "parts": [{
                        "text": r#"{"name":"List orders","method":"GET","url":"https://api.example.com/orders","headers":[],"query_params":[],"body":null,"description":"Fetch all orders"}"#
                    }]
                },
                "finishReason": "STOP"
            }]
        })
        .to_string();

        let port = spawn_mock_server_with_status("200 OK", &gemini_style_body);
        let provider = GoogleProvider::with_base_url(
            Client::new(),
            "test-key",
            &format!("http://127.0.0.1:{port}"),
        );

        let def = provider.generate_api("get all orders").await.unwrap();
        assert_eq!(def.name, "List orders");
        assert_eq!(def.method, "GET");
    }

    #[tokio::test]
    async fn google_surfaces_non_success_status_as_ai_error() {
        let port = spawn_mock_server_with_status("403 Forbidden", "{\"error\":\"bad key\"}");
        let provider = GoogleProvider::with_base_url(
            Client::new(),
            "bad-key",
            &format!("http://127.0.0.1:{port}"),
        );

        let err = provider.generate_api("hello").await.unwrap_err();
        match err {
            AppError::Ai(msg) => assert!(msg.contains("403")),
            other => panic!("expected AppError::Ai, got {other:?}"),
        }
    }
}
