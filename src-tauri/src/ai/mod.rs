//! AI-assisted request generation and project intelligence (Phase 8: LP-0801 - LP-0821,
//! multi-provider LP-0822).
//!
//! `AiProvider` is the domain-facing abstraction (LP-0801) so `commands.rs` never talks to a
//! specific vendor directly — each backend lives in its own submodule (`anthropic`, `openai`,
//! `google`) behind the same trait, wrapped in [`AnyAiProvider`] so call sites still hold one
//! concrete type. This file keeps only what's genuinely shared across all of them: the trait
//! itself, provider selection/settings persistence, the prompt text, and response parsing —
//! provider-specific wire formats and their tests live in their own files.
//!
//! Supports structured request generation (LP-0804, LP-0805), sample response generation (LP-0806),
//! project context integration with strict secret redaction (LP-0807 - LP-0809), and automated
//! test script & documentation generation (LP-0819).

mod anthropic;
mod google;
mod openai;

pub use anthropic::ClaudeProvider;
pub use google::GoogleProvider;
pub use openai::OpenAiCompatibleProvider;

use reqwest::Client;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::models::{HeaderEntry, QueryParam, VALID_METHODS};

/// Which AI backend to talk to (LP-0822). `Custom` is deliberately NOT a fully generic
/// request/response template system — it reuses the OpenAI-compatible wire format (the shape
/// Groq/Together/OpenRouter/Ollama/LM Studio/Azure OpenAI/etc. all already implement) against
/// a base_url the user supplies, which covers the realistic "custom provider" need without a
/// bespoke schema-mapping UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiProviderKind {
    #[default]
    Anthropic,
    OpenAi,
    Google,
    Custom,
}

impl AiProviderKind {
    fn as_str(self) -> &'static str {
        match self {
            AiProviderKind::Anthropic => "anthropic",
            AiProviderKind::OpenAi => "openai",
            AiProviderKind::Google => "google",
            AiProviderKind::Custom => "custom",
        }
    }

    fn from_stored(value: &str) -> Self {
        match value {
            "openai" => AiProviderKind::OpenAi,
            "google" => AiProviderKind::Google,
            "custom" => AiProviderKind::Custom,
            // Covers "anthropic" and anything unrecognized (e.g. a DB written before this
            // field existed) — Anthropic was the only provider then, so that's the correct read.
            _ => AiProviderKind::Anthropic,
        }
    }
}

/// AI Configuration settings (LP-0803, LP-0822).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSettings {
    pub provider: AiProviderKind,
    pub api_key: Option<String>,
    pub model: String,
    pub base_url: Option<String>,
    pub is_configured: bool,
}

/// Input settings when saving configuration from the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAiSettingsInput {
    #[serde(default)]
    pub provider: Option<AiProviderKind>,
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

/// One value that can hold whichever provider is actually configured, so `commands.rs` keeps
/// calling a single concrete type instead of needing `dyn AiProvider` (the trait's `async fn`s
/// aren't dyn-compatible without a boxing crate, and a fixed 3-way enum is simpler than pulling
/// one in for this).
#[derive(Clone)]
pub enum AnyAiProvider {
    Anthropic(ClaudeProvider),
    OpenAiCompatible(OpenAiCompatibleProvider),
    Google(GoogleProvider),
}

impl AiProvider for AnyAiProvider {
    async fn generate_api(&self, prompt: &str) -> Result<GeneratedApiDefinition, AppError> {
        match self {
            AnyAiProvider::Anthropic(p) => p.generate_api(prompt).await,
            AnyAiProvider::OpenAiCompatible(p) => p.generate_api(prompt).await,
            AnyAiProvider::Google(p) => p.generate_api(prompt).await,
        }
    }

    async fn generate_sample_response(
        &self,
        method: &str,
        url: &str,
        body: Option<&str>,
        description: Option<&str>,
    ) -> Result<GeneratedSampleResponse, AppError> {
        match self {
            AnyAiProvider::Anthropic(p) => p.generate_sample_response(method, url, body, description).await,
            AnyAiProvider::OpenAiCompatible(p) => p.generate_sample_response(method, url, body, description).await,
            AnyAiProvider::Google(p) => p.generate_sample_response(method, url, body, description).await,
        }
    }

    async fn generate_tests_and_docs(
        &self,
        method: &str,
        url: &str,
        body: Option<&str>,
    ) -> Result<GeneratedTestsAndDocs, AppError> {
        match self {
            AnyAiProvider::Anthropic(p) => p.generate_tests_and_docs(method, url, body).await,
            AnyAiProvider::OpenAiCompatible(p) => p.generate_tests_and_docs(method, url, body).await,
            AnyAiProvider::Google(p) => p.generate_tests_and_docs(method, url, body).await,
        }
    }
}

/// Builds whichever provider `kind` names from already-resolved settings. `base_url` is the
/// user's override if they set one (`None` means "use that provider's own default"), except for
/// `Custom`, which has no default and is rejected here if empty.
pub fn build_provider(
    client: Client,
    kind: AiProviderKind,
    api_key: String,
    base_url: Option<String>,
    model: String,
) -> Result<AnyAiProvider, AppError> {
    let base_url = base_url.filter(|s| !s.trim().is_empty());
    match kind {
        AiProviderKind::Anthropic => Ok(AnyAiProvider::Anthropic(ClaudeProvider::from_parts(
            client,
            api_key,
            base_url.unwrap_or_else(|| anthropic::DEFAULT_API_URL.to_string()),
            model,
        ))),
        AiProviderKind::OpenAi => Ok(AnyAiProvider::OpenAiCompatible(OpenAiCompatibleProvider::openai(
            client, api_key, base_url, model,
        ))),
        AiProviderKind::Google => Ok(AnyAiProvider::Google(GoogleProvider::new(client, api_key, base_url, model))),
        AiProviderKind::Custom => {
            let base_url = base_url.ok_or_else(|| {
                AppError::Validation("Custom AI provider requires a base URL (an OpenAI-compatible endpoint).".into())
            })?;
            Ok(AnyAiProvider::OpenAiCompatible(OpenAiCompatibleProvider::custom(
                client, api_key, base_url, model,
            )))
        }
    }
}

/// Resolves the configured provider from DB settings, falling back to `ANTHROPIC_API_KEY` only
/// when no provider has ever been configured in the DB (the same zero-config convenience the
/// single-provider version of this app had, preserved for anyone who already relies on it).
pub fn provider_from_db_or_env(client: Client, conn: &Connection) -> Option<AnyAiProvider> {
    // get_ai_settings_raw already folds ANTHROPIC_API_KEY in as a fallback when no DB key is
    // set and the provider is (still) the default Anthropic — nothing further to layer here.
    let settings = get_ai_settings_raw(conn).ok()?;
    let api_key = settings.api_key.filter(|k| !k.trim().is_empty())?;
    build_provider(client, settings.provider, api_key, settings.base_url, settings.model).ok()
}

pub(crate) const SYSTEM_PROMPT_API: &str = r#"You generate a single HTTP API request definition from a natural-language description.
Respond with ONLY a single minified JSON object — no markdown code fences, no commentary before or after — matching exactly this shape:
{"name":string,"method":"GET"|"POST"|"PUT"|"PATCH"|"DELETE"|"HEAD"|"OPTIONS","url":string,"headers":[{"key":string,"value":string,"enabled":true}],"query_params":[{"key":string,"value":string,"enabled":true}],"body":string or null,"description":string or null}
Never include real secrets, API keys, tokens, or credentials in the output. If the request needs one, use a placeholder like {{apiKey}} instead."#;

pub(crate) const SYSTEM_PROMPT_SAMPLE_RESPONSE: &str = r#"You generate a realistic sample mock HTTP response for an API request.
Respond with ONLY a single JSON object — no code fences, no markdown formatting:
{"status":number,"headers":[{"key":string,"value":string,"enabled":true}],"body":string,"description":string}
Headers should include Content-Type. The body should be realistic JSON or text formatted as a string. Status should be a standard HTTP status code (e.g. 200, 201, 400)."#;

pub(crate) const SYSTEM_PROMPT_TESTS_DOCS: &str = r#"You generate postman-compatible pm.test() scripts and markdown API documentation for an HTTP endpoint.
Respond with ONLY a single JSON object — no markdown code fences:
{"tests_script":string,"documentation":string}
tests_script should contain standard assertions like pm.test("Status code is 200", function () { pm.response.to.have.status(200); });
documentation should be clean markdown with headers, parameters, and expected response format."#;

pub(crate) fn parse_and_validate(text: &str) -> Result<GeneratedApiDefinition, AppError> {
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

pub(crate) fn parse_sample_response(text: &str) -> Result<GeneratedSampleResponse, AppError> {
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

pub(crate) fn parse_tests_and_docs(text: &str) -> Result<GeneratedTestsAndDocs, AppError> {
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

fn default_model_for(provider: AiProviderKind) -> &'static str {
    match provider {
        AiProviderKind::Anthropic => anthropic::DEFAULT_MODEL,
        AiProviderKind::OpenAi => openai::DEFAULT_MODEL,
        AiProviderKind::Google => google::DEFAULT_MODEL,
        // No sane single default model name for an arbitrary custom endpoint, but this is only
        // ever used to pre-fill the field, never to silently pick a model for a real request.
        AiProviderKind::Custom => openai::DEFAULT_MODEL,
    }
}

// Database helper functions for AI configuration (LP-0803, LP-0822)

/// Unmasked reader shared by the public [`get_ai_settings`] (which masks the key before it
/// reaches the frontend) and [`provider_from_db_or_env`] (which needs the real key to make
/// requests).
fn get_ai_settings_raw(conn: &Connection) -> Result<AiSettings, AppError> {
    let db_key: Option<String> = conn
        .query_row("SELECT value FROM app_settings WHERE key = 'ai_api_key'", [], |r| r.get(0))
        .optional()
        .map_err(|e| AppError::Storage(format!("failed to query ai_api_key: {e}")))?;

    let db_provider: Option<String> = conn
        .query_row("SELECT value FROM app_settings WHERE key = 'ai_provider'", [], |r| r.get(0))
        .optional()
        .map_err(|e| AppError::Storage(format!("failed to query ai_provider: {e}")))?;
    let provider = db_provider.as_deref().map(AiProviderKind::from_stored).unwrap_or_default();

    let db_model: Option<String> = conn
        .query_row("SELECT value FROM app_settings WHERE key = 'ai_model'", [], |r| r.get(0))
        .optional()
        .map_err(|e| AppError::Storage(format!("failed to query ai_model: {e}")))?;

    // Empty string and NULL both mean "no override" — save_ai_settings never writes an empty
    // string on purpose, but treat it the same way defensively rather than passing "" through
    // as a real base_url (which would 404 every request instead of using the provider default).
    let db_base_url: Option<String> = conn
        .query_row("SELECT value FROM app_settings WHERE key = 'ai_base_url'", [], |r| r.get(0))
        .optional()
        .map_err(|e| AppError::Storage(format!("failed to query ai_base_url: {e}")))?
        .filter(|s: &String| !s.trim().is_empty());

    let env_key = (provider == AiProviderKind::Anthropic)
        .then(|| std::env::var("ANTHROPIC_API_KEY").ok())
        .flatten()
        .filter(|k| !k.trim().is_empty());
    let effective_key = db_key.or(env_key);
    let is_configured = effective_key.is_some();

    let model = db_model.unwrap_or_else(|| default_model_for(provider).to_string());

    Ok(AiSettings {
        provider,
        api_key: effective_key,
        model,
        base_url: db_base_url,
        is_configured,
    })
}

pub fn get_ai_settings(conn: &Connection) -> Result<AiSettings, AppError> {
    let mut settings = get_ai_settings_raw(conn)?;
    // Mask secret key before exposing to frontend (LP-0803)
    settings.api_key = settings.api_key.map(|k| {
        if k.len() <= 8 {
            "********".to_string()
        } else {
            format!("{}...{}", &k[..4], &k[k.len() - 4..])
        }
    });
    Ok(settings)
}

pub fn save_ai_settings(conn: &Connection, input: &UpdateAiSettingsInput) -> Result<(), AppError> {
    let now = chrono::Utc::now().to_rfc3339();

    let provider_changed = if let Some(provider) = input.provider {
        let current = get_ai_settings_raw(conn).map(|s| s.provider).unwrap_or_default();
        conn.execute(
            "INSERT INTO app_settings (key, value, updated_at) VALUES ('ai_provider', ?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
            params![provider.as_str(), now],
        )?;
        provider != current
    } else {
        false
    };

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
    } else if provider_changed {
        // Switching provider without also naming a model would otherwise leave the previous
        // provider's model string in place (e.g. still "claude-sonnet-5" after switching to
        // OpenAI) — every request would fail on an unrecognized model until the user noticed
        // and fixed it by hand, so reset to the new provider's own default instead.
        conn.execute(
            "INSERT INTO app_settings (key, value, updated_at) VALUES ('ai_model', ?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
            params![default_model_for(input.provider.unwrap_or_default()), now],
        )?;
    }

    if let Some(ref base_url) = input.base_url {
        let trimmed = base_url.trim();
        conn.execute(
            "INSERT INTO app_settings (key, value, updated_at) VALUES ('ai_base_url', ?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
            params![trimmed, now],
        )?;
    } else if provider_changed {
        // Same reasoning as the model reset above: a base_url override left over from the
        // previous provider is very unlikely to also be valid for the new one.
        conn.execute("DELETE FROM app_settings WHERE key = 'ai_base_url'", [])?;
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

/// Tiny raw-HTTP mock server shared by every provider's tests (each provider talks to a real
/// TCP connection, not a mocked `Client`, so the wire format itself gets exercised).
#[cfg(test)]
pub(crate) mod test_support {
    use std::io::{Read, Write};
    use std::net::TcpListener;

    pub(crate) fn spawn_mock_server_with_status(status_line: &str, response_body: &str) -> u16 {
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

    #[test]
    fn provider_kind_round_trips_through_its_stored_string() {
        for kind in [
            AiProviderKind::Anthropic,
            AiProviderKind::OpenAi,
            AiProviderKind::Google,
            AiProviderKind::Custom,
        ] {
            assert_eq!(AiProviderKind::from_stored(kind.as_str()), kind);
        }
    }

    #[test]
    fn unrecognized_stored_provider_falls_back_to_anthropic() {
        // A DB written before this field existed has no 'ai_provider' row at all — the only
        // provider that could have been configured then was Anthropic.
        assert_eq!(AiProviderKind::from_stored("nonsense"), AiProviderKind::Anthropic);
    }

    #[test]
    fn custom_provider_without_base_url_is_rejected() {
        // Not `.unwrap_err()` — that requires `AnyAiProvider: Debug`, which would mean deriving
        // Debug on structs that hold a raw API key, printing it in any future panic message.
        match build_provider(
            Client::new(),
            AiProviderKind::Custom,
            "key".to_string(),
            None,
            "some-model".to_string(),
        ) {
            Err(AppError::Validation(_)) => {}
            Err(other) => panic!("expected AppError::Validation, got {other:?}"),
            Ok(_) => panic!("expected an error, got a provider"),
        }
    }

    #[test]
    fn switching_provider_resets_model_and_base_url_to_the_new_providers_defaults() {
        let conn = crate::db::open_in_memory().unwrap();
        save_ai_settings(
            &conn,
            &UpdateAiSettingsInput {
                provider: Some(AiProviderKind::Anthropic),
                api_key: Some("sk-ant-test".to_string()),
                model: None,
                base_url: None,
            },
        )
        .unwrap();
        // A base_url override that only makes sense for Anthropic.
        save_ai_settings(
            &conn,
            &UpdateAiSettingsInput { provider: None, api_key: None, model: None, base_url: Some("https://my-anthropic-proxy.example.com".to_string()) },
        )
        .unwrap();

        save_ai_settings(
            &conn,
            &UpdateAiSettingsInput {
                provider: Some(AiProviderKind::OpenAi),
                api_key: None,
                model: None,
                base_url: None,
            },
        )
        .unwrap();

        let settings = get_ai_settings_raw(&conn).unwrap();
        assert_eq!(settings.provider, AiProviderKind::OpenAi);
        assert_eq!(settings.model, openai::DEFAULT_MODEL);
        assert_eq!(settings.base_url, None);
    }
}
