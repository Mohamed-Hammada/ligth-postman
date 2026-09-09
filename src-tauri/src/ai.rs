//! AI-assisted request generation (README Phase 8 / task pack LP-0801-0805).
//!
//! `AiProvider` is the domain-facing abstraction (LP-0801) so `commands.rs` never talks to
//! Claude directly — swapping providers later means a new impl, not touching call sites.
//! `ClaudeProvider` (LP-0802) is the only implementation today, calling the Anthropic
//! Messages API over raw HTTP: Rust has no official Anthropic SDK, so per the API skill's
//! own rule ("SDK when one exists, raw HTTP only when it doesn't") this is the correct
//! approach here, not a shortcut.
//!
//! Scope boundary (documented, not hidden): this module only ever sends the user's typed
//! prompt to the model. It does not read project data, requests, variables, or secrets —
//! there is nothing here yet that LP-0809's redaction requirement needs to redact, because
//! nothing project-derived is sent. Source-project context sharing (LP-0808/0810+) is not
//! implemented.

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::models::{HeaderEntry, QueryParam, VALID_METHODS};

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";
const CLAUDE_MODEL: &str = "claude-opus-5";

/// Structured output (LP-0804) — the caller always gets this typed shape, never raw text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedApiDefinition {
    pub name: String,
    pub method: String,
    pub url: String,
    #[serde(default)]
    pub headers: Vec<HeaderEntry>,
    #[serde(default)]
    pub query_params: Vec<QueryParam>,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

pub trait AiProvider {
    async fn generate_api(&self, prompt: &str) -> Result<GeneratedApiDefinition, AppError>;
}

pub struct ClaudeProvider {
    client: Client,
    api_key: String,
    api_url: String,
}

impl ClaudeProvider {
    /// Returns `None` (not an error) when no key is configured — the caller decides how to
    /// surface that (LP-0803: AI is an opt-in feature, its absence isn't a startup failure).
    pub fn from_env(client: Client) -> Option<Self> {
        std::env::var("ANTHROPIC_API_KEY")
            .ok()
            .filter(|key| !key.trim().is_empty())
            .map(|api_key| Self { client, api_key, api_url: ANTHROPIC_API_URL.to_string() })
    }

    /// Test-only: points at a local mock server instead of the real Anthropic API so the
    /// full request/response/parse pipeline can be verified against real HTTP, not just
    /// the parsing function in isolation.
    #[cfg(test)]
    fn with_base_url(client: Client, api_key: &str, api_url: &str) -> Self {
        Self { client, api_key: api_key.to_string(), api_url: api_url.to_string() }
    }
}

impl AiProvider for ClaudeProvider {
    async fn generate_api(&self, prompt: &str) -> Result<GeneratedApiDefinition, AppError> {
        let trimmed_prompt = prompt.trim();
        if trimmed_prompt.is_empty() {
            return Err(AppError::Validation("AI prompt must not be empty".into()));
        }

        let request_body = serde_json::json!({
            "model": CLAUDE_MODEL,
            "max_tokens": 2048,
            "system": SYSTEM_PROMPT,
            "messages": [{ "role": "user", "content": trimmed_prompt }],
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
            // Never echo response headers/body verbatim into logs — could contain
            // account-identifying info. The error string only carries what the frontend
            // needs to show the user.
            let text = response.text().await.unwrap_or_default();
            return Err(AppError::Ai(format!("Claude API returned {status}: {text}")));
        }

        let parsed: AnthropicMessageResponse = response
            .json()
            .await
            .map_err(|err| AppError::Ai(format!("failed to parse Claude API response: {err}")))?;

        let text = parsed
            .content
            .into_iter()
            .find_map(|block| (block.block_type == "text").then_some(block.text))
            .ok_or_else(|| AppError::Ai("Claude response contained no text block".into()))?;

        parse_and_validate(&text)
    }
}

const SYSTEM_PROMPT: &str = r#"You generate a single HTTP API request definition from a natural-language description.
Respond with ONLY a single minified JSON object — no markdown code fences, no commentary before or after — matching exactly this shape:
{"name":string,"method":"GET"|"POST"|"PUT"|"PATCH"|"DELETE"|"HEAD"|"OPTIONS","url":string,"headers":[{"key":string,"value":string,"enabled":true}],"query_params":[{"key":string,"value":string,"enabled":true}],"body":string or null,"description":string or null}
Never include real secrets, API keys, tokens, or credentials in the output. If the request needs one, use a placeholder like {{apiKey}} instead."#;

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

/// Structured-output validation (LP-0805 "structured API -> validation -> preview"): reject
/// a malformed or unsafe generation here rather than trusting model output verbatim.
fn parse_and_validate(text: &str) -> Result<GeneratedApiDefinition, AppError> {
    let cleaned = text
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    let definition: GeneratedApiDefinition = serde_json::from_str(cleaned)
        .map_err(|err| AppError::Ai(format!("AI returned invalid JSON: {err}")))?;

    if definition.name.trim().is_empty() {
        return Err(AppError::Ai("AI generation is missing a name".into()));
    }
    let method = definition.method.trim().to_uppercase();
    if !VALID_METHODS.contains(&method.as_str()) {
        return Err(AppError::Ai(format!("AI generated an unsupported method '{method}'")));
    }
    if definition.url.trim().is_empty() {
        return Err(AppError::Ai("AI generation is missing a URL".into()));
    }

    Ok(GeneratedApiDefinition { method, ..definition })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_clean_json() {
        let result = parse_and_validate(
            r#"{"name":"Get users","method":"get","url":"https://api.example.com/users","headers":[],"query_params":[],"body":null,"description":null}"#,
        )
        .unwrap();
        assert_eq!(result.method, "GET"); // normalized
        assert_eq!(result.name, "Get users");
    }

    #[test]
    fn strips_markdown_code_fence() {
        let wrapped = format!(
            "```json\n{}\n```",
            r#"{"name":"Ping","method":"GET","url":"https://api.example.com/ping"}"#
        );
        let result = parse_and_validate(&wrapped).unwrap();
        assert_eq!(result.name, "Ping");
    }

    #[test]
    fn rejects_invalid_json() {
        let result = parse_and_validate("not json at all");
        assert!(matches!(result, Err(AppError::Ai(_))));
    }

    #[test]
    fn rejects_unsupported_method() {
        let result = parse_and_validate(
            r#"{"name":"Weird","method":"TRACE","url":"https://api.example.com"}"#,
        );
        assert!(matches!(result, Err(AppError::Ai(_))));
    }

    #[test]
    fn rejects_missing_url() {
        let result = parse_and_validate(r#"{"name":"Broken","method":"GET","url":""}"#);
        assert!(matches!(result, Err(AppError::Ai(_))));
    }

    use std::io::{Read, Write};
    use std::net::TcpListener;

    fn spawn_mock_anthropic_server(response_body: &str) -> u16 {
        let mut response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            response_body.len()
        );
        response.push_str(response_body);
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock server");
        let port = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let _ = stream.set_nodelay(true);
                let mut received = Vec::new();
                let mut buf = [0u8; 4096];
                loop {
                    match stream.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            received.extend_from_slice(&buf[..n]);
                            if received.windows(4).any(|w| w == b"\r\n\r\n") {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
            }
        });
        port
    }

    #[tokio::test]
    async fn generate_api_end_to_end_against_a_real_http_response() {
        let anthropic_style_body = serde_json::json!({
            "id": "msg_test",
            "type": "message",
            "role": "assistant",
            "content": [{
                "type": "text",
                "text": r#"{"name":"Get weather","method":"get","url":"https://api.example.com/weather?city={{city}}","headers":[],"query_params":[],"body":null,"description":"Fetch current weather for a city"}"#
            }],
            "model": "claude-opus-5",
            "stop_reason": "end_turn",
        })
        .to_string();
        let port = spawn_mock_anthropic_server(&anthropic_style_body);

        let provider = ClaudeProvider::with_base_url(
            Client::new(),
            "test-key",
            &format!("http://127.0.0.1:{port}/v1/messages"),
        );

        let result = provider.generate_api("get the weather for a city").await.unwrap();

        assert_eq!(result.method, "GET");
        assert_eq!(result.name, "Get weather");
        assert_eq!(result.url, "https://api.example.com/weather?city={{city}}");
    }

    #[tokio::test]
    async fn generate_api_surfaces_non_success_status_as_ai_error() {
        let error_body = r#"{"type":"error","error":{"type":"authentication_error","message":"invalid x-api-key"}}"#;
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock server");
        let port = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 4096];
                let _ = stream.read(&mut buf);
                let response = format!(
                    "HTTP/1.1 401 Unauthorized\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    error_body.len(),
                    error_body
                );
                let _ = stream.write_all(response.as_bytes());
            }
        });

        let provider = ClaudeProvider::with_base_url(
            Client::new(),
            "bad-key",
            &format!("http://127.0.0.1:{port}/v1/messages"),
        );

        let result = provider.generate_api("anything").await;
        assert!(matches!(result, Err(AppError::Ai(_))));
    }

    #[tokio::test]
    async fn empty_prompt_is_rejected_without_a_network_call() {
        // No mock server is even started — if this reached the network it would hang
        // trying to connect (port 1 refuses instantly, but a real endpoint could hang),
        // not return quickly with a Validation error.
        let provider = ClaudeProvider::with_base_url(Client::new(), "key", "http://127.0.0.1:1/v1/messages");
        let result = provider.generate_api("   ").await;
        assert!(matches!(result, Err(AppError::Validation(_))));
    }
}
