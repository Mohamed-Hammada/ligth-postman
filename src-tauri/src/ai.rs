//! AI-assisted request generation and project intelligence (Phase 8: LP-0801 - LP-0821).
//!
//! `AiProvider` is the domain-facing abstraction (LP-0801) so `commands.rs` never talks to
//! Claude directly — swapping providers later means a new impl, not touching call sites.
//! `ClaudeProvider` (LP-0802) calls the Anthropic Messages API over raw HTTP with secure
//! key storage and model configuration (LP-0803).
//!
//! Supports structured request generation (LP-0804, LP-0805), sample response generation (LP-0806),
//! project context integration with strict secret redaction (LP-0807 - LP-0809), and automated
//! test script & documentation generation (LP-0819).

use reqwest::Client;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::models::{HeaderEntry, QueryParam, VALID_METHODS};

pub const DEFAULT_ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
pub const DEFAULT_CLAUDE_MODEL: &str = "claude-3-5-sonnet-20241022";
pub const ANTHROPIC_VERSION: &str = "2023-06-01";

/// AI Configuration settings (LP-0803).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSettings {
    pub api_key: Option<String>,
    pub model: String,
    pub base_url: Option<String>,
    pub is_configured: bool,
}

/// Input settings when saving configuration from the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAiSettingsInput {
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub base_url: Option<String>,
}

/// Structured output for generated requests (LP-0804).
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

/// Structured output for generated sample responses (LP-0806).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedSampleResponse {
    pub status: u16,
    #[serde(default)]
    pub headers: Vec<HeaderEntry>,
    pub body: String,
    pub description: Option<String>,
}

/// Structured output for generated test assertions and documentation (LP-0819).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedTestsAndDocs {
    pub tests_script: String,
    pub documentation: String,
}

/// Context summary for project-level Ask AI (LP-0807, LP-0808, LP-0809).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectAiContext {
    pub project_name: String,
    pub existing_endpoints: Vec<String>,
    pub variable_keys: Vec<String>, // strictly keys only, NO values or secrets!
}

#[allow(async_fn_in_trait)]
pub trait AiProvider {
    async fn generate_api(&self, prompt: &str) -> Result<GeneratedApiDefinition, AppError>;
    async fn generate_sample_response(
        &self,
        method: &str,
        url: &str,
        body: Option<&str>,
        description: Option<&str>,
    ) -> Result<GeneratedSampleResponse, AppError>;
    async fn generate_tests_and_docs(
        &self,
        method: &str,
        url: &str,
        body: Option<&str>,
    ) -> Result<GeneratedTestsAndDocs, AppError>;
}

#[derive(Clone)]
pub struct ClaudeProvider {
    pub(crate) client: Client,
    pub(crate) api_key: String,
    pub(crate) api_url: String,
    pub(crate) model: String,
}

impl ClaudeProvider {
    #[allow(dead_code)]
    pub fn new(client: Client, api_key: String, api_url: String, model: String) -> Self {
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
                api_url: DEFAULT_ANTHROPIC_API_URL.to_string(),
                model: DEFAULT_CLAUDE_MODEL.to_string(),
            })
    }

    /// Build provider using DB settings with fallback to environment variables (LP-0803).
    pub fn from_db_or_env(client: Client, conn: &Connection) -> Option<Self> {
        let db_key: Option<String> = conn
            .query_row("SELECT value FROM app_settings WHERE key = 'ai_api_key'", [], |r| r.get(0))
            .optional()
            .ok()
            .flatten();

        let api_key = db_key
            .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
            .filter(|k| !k.trim().is_empty())?;

        let model: String = conn
            .query_row("SELECT value FROM app_settings WHERE key = 'ai_model'", [], |r| r.get(0))
            .optional()
            .ok()
            .flatten()
            .unwrap_or_else(|| DEFAULT_CLAUDE_MODEL.to_string());

        let api_url: String = conn
            .query_row("SELECT value FROM app_settings WHERE key = 'ai_base_url'", [], |r| r.get(0))
            .optional()
            .ok()
            .flatten()
            .unwrap_or_else(|| DEFAULT_ANTHROPIC_API_URL.to_string());

        Some(Self { client, api_key, api_url, model })
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

const SYSTEM_PROMPT_API: &str = r#"You generate a single HTTP API request definition from a natural-language description.
Respond with ONLY a single minified JSON object — no markdown code fences, no commentary before or after — matching exactly this shape:
{"name":string,"method":"GET"|"POST"|"PUT"|"PATCH"|"DELETE"|"HEAD"|"OPTIONS","url":string,"headers":[{"key":string,"value":string,"enabled":true}],"query_params":[{"key":string,"value":string,"enabled":true}],"body":string or null,"description":string or null}
Never include real secrets, API keys, tokens, or credentials in the output. If the request needs one, use a placeholder like {{apiKey}} instead."#;

const SYSTEM_PROMPT_SAMPLE_RESPONSE: &str = r#"You generate a realistic sample mock HTTP response for an API request.
Respond with ONLY a single JSON object — no code fences, no markdown formatting:
{"status":number,"headers":[{"key":string,"value":string,"enabled":true}],"body":string,"description":string}
Headers should include Content-Type. The body should be realistic JSON or text formatted as a string. Status should be a standard HTTP status code (e.g. 200, 201, 400)."#;

const SYSTEM_PROMPT_TESTS_DOCS: &str = r#"You generate postman-compatible pm.test() scripts and markdown API documentation for an HTTP endpoint.
Respond with ONLY a single JSON object — no markdown code fences:
{"tests_script":string,"documentation":string}
tests_script should contain standard assertions like pm.test("Status code is 200", function () { pm.response.to.have.status(200); });
documentation should be clean markdown with headers, parameters, and expected response format."#;

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

fn parse_sample_response(text: &str) -> Result<GeneratedSampleResponse, AppError> {
    let cleaned = text
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    let sample: GeneratedSampleResponse = serde_json::from_str(cleaned)
        .map_err(|err| AppError::Ai(format!("AI returned invalid sample response JSON: {err}")))?;

    if !(100..=599).contains(&sample.status) {
        return Err(AppError::Ai(format!("AI generated invalid HTTP status code {}", sample.status)));
    }

    Ok(sample)
}

fn parse_tests_and_docs(text: &str) -> Result<GeneratedTestsAndDocs, AppError> {
    let cleaned = text
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    let result: GeneratedTestsAndDocs = serde_json::from_str(cleaned)
        .map_err(|err| AppError::Ai(format!("AI returned invalid tests and docs JSON: {err}")))?;

    Ok(result)
}

// Database helper functions for AI configuration (LP-0803)
pub fn get_ai_settings(conn: &Connection) -> Result<AiSettings, AppError> {
    let db_key: Option<String> = conn
        .query_row("SELECT value FROM app_settings WHERE key = 'ai_api_key'", [], |r| r.get(0))
        .optional()
        .map_err(|e| AppError::Storage(format!("failed to query ai_api_key: {e}")))?;

    let db_model: Option<String> = conn
        .query_row("SELECT value FROM app_settings WHERE key = 'ai_model'", [], |r| r.get(0))
        .optional()
        .map_err(|e| AppError::Storage(format!("failed to query ai_model: {e}")))?;

    let db_base_url: Option<String> = conn
        .query_row("SELECT value FROM app_settings WHERE key = 'ai_base_url'", [], |r| r.get(0))
        .optional()
        .map_err(|e| AppError::Storage(format!("failed to query ai_base_url: {e}")))?;

    let env_key = std::env::var("ANTHROPIC_API_KEY").ok().filter(|k| !k.trim().is_empty());
    let effective_key = db_key.or(env_key);
    let is_configured = effective_key.is_some();

    // Mask secret key before exposing to frontend (LP-0803)
    let masked_key = effective_key.map(|k| {
        if k.len() <= 8 {
            "********".to_string()
        } else {
            format!("{}...{}", &k[..4], &k[k.len() - 4..])
        }
    });

    let model = db_model.unwrap_or_else(|| DEFAULT_CLAUDE_MODEL.to_string());

    Ok(AiSettings {
        api_key: masked_key,
        model,
        base_url: db_base_url,
        is_configured,
    })
}

pub fn save_ai_settings(conn: &Connection, input: &UpdateAiSettingsInput) -> Result<(), AppError> {
    let now = chrono::Utc::now().to_rfc3339();

    if let Some(ref key) = input.api_key {
        let trimmed = key.trim();
        // Do not overwrite with masked key e.g. "sk-a...1234"
        if !trimmed.is_empty() && !trimmed.contains("...") {
            conn.execute(
                "INSERT INTO app_settings (key, value, updated_at) VALUES ('ai_api_key', ?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
                params![trimmed, now],
            )?;
        }
    }

    if let Some(ref model) = input.model {
        let trimmed = model.trim();
        if !trimmed.is_empty() {
            conn.execute(
                "INSERT INTO app_settings (key, value, updated_at) VALUES ('ai_model', ?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
                params![trimmed, now],
            )?;
        }
    }

    if let Some(ref base_url) = input.base_url {
        let trimmed = base_url.trim();
        conn.execute(
            "INSERT INTO app_settings (key, value, updated_at) VALUES ('ai_base_url', ?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
            params![trimmed, now],
        )?;
    }

    Ok(())
}

/// Builds project context with strict secret redaction (LP-0808, LP-0809).
pub fn build_project_context_prompt(
    user_prompt: &str,
    context: &ProjectAiContext,
) -> String {
    let mut prompt = format!("Project Name: {}\n", context.project_name);
    if !context.existing_endpoints.is_empty() {
        prompt.push_str("Existing Endpoints in Project:\n");
        for ep in &context.existing_endpoints {
            prompt.push_str(&format!("- {ep}\n"));
        }
    }
    if !context.variable_keys.is_empty() {
        prompt.push_str("Available Variable Names (placeholder keys only):\n");
        for key in &context.variable_keys {
            prompt.push_str(&format!("- {{{{ {key} }}}}\n"));
        }
    }
    prompt.push_str("\nUser Request:\n");
    prompt.push_str(user_prompt);
    prompt
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
        assert_eq!(result.method, "GET");
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
            r#"{"name":"Weird","method":"CONNECT","url":"https://api.example.com"}"#,
        );
        assert!(matches!(result, Err(AppError::Ai(_))));
    }

    #[test]
    fn rejects_missing_url() {
        let result = parse_and_validate(r#"{"name":"Broken","method":"GET","url":""}"#);
        assert!(matches!(result, Err(AppError::Ai(_))));
    }

    #[test]
    fn parses_sample_response_json() {
        let json = r#"{"status":200,"headers":[{"key":"Content-Type","value":"application/json","enabled":true}],"body":"{\"success\":true}","description":"OK response"}"#;
        let sample = parse_sample_response(json).unwrap();
        assert_eq!(sample.status, 200);
        assert_eq!(sample.headers.len(), 1);
        assert_eq!(sample.description.as_deref(), Some("OK response"));
    }

    #[test]
    fn parses_tests_and_docs_json() {
        let json = r##"{"tests_script":"pm.test('ok', function() {});","documentation":"# API Doc"}"##;
        let td = parse_tests_and_docs(json).unwrap();
        assert!(td.tests_script.contains("pm.test"));
        assert_eq!(td.documentation, "# API Doc");
    }

    #[test]
    fn project_context_omits_secrets_and_formats_cleanly() {
        let ctx = ProjectAiContext {
            project_name: "Customer API".to_string(),
            existing_endpoints: vec!["GET /users".to_string(), "POST /users".to_string()],
            variable_keys: vec!["baseUrl".to_string(), "apiVersion".to_string()],
        };

        let prompt = build_project_context_prompt("Create endpoint to update user", &ctx);
        assert!(prompt.contains("Customer API"));
        assert!(prompt.contains("GET /users"));
        assert!(prompt.contains("{{ baseUrl }}"));
        assert!(prompt.contains("Create endpoint to update user"));
    }

    use std::io::{Read, Write};
    use std::net::TcpListener;

    fn spawn_mock_server_with_status(status_line: &str, response_body: &str) -> u16 {
        let mut response = format!(
            "HTTP/1.1 {status_line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
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
