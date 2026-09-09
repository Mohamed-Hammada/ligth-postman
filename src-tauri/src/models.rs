use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Methods we accept today. README §13 will grow this alongside auth/scripts later.
pub const VALID_METHODS: &[&str] = &[
    "GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewProjectInput {
    pub name: String,
}

/// `name: None` means "leave unchanged" — omitted fields must never clobber existing data.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateProjectInput {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderEntry {
    pub key: String,
    pub value: String,
    pub enabled: bool,
}

/// Query parameters are kept separate from `url` (never baked in) so serialization is
/// runtime-only — a disabled param, or a param whose value resolves from a variable, must
/// be re-derivable every time without ever rewriting the stored request (README §41 style).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryParam {
    pub key: String,
    pub value: String,
    pub enabled: bool,
    #[serde(default)]
    pub description: Option<String>,
}

/// Lightweight row for lists — never carries headers/body (README §4/§20 lazy loading).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestSummary {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub method: String,
    pub url: String,
    pub updated_at: DateTime<Utc>,
}

/// Fully hydrated request — only fetched when a tab is actually activated (README §4/§5).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestFull {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub method: String,
    pub url: String,
    pub headers: Vec<HeaderEntry>,
    pub query_params: Vec<QueryParam>,
    pub body: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewRequestInput {
    pub project_id: String,
    pub name: String,
    pub method: String,
    pub url: String,
    #[serde(default)]
    pub headers: Vec<HeaderEntry>,
    #[serde(default)]
    pub query_params: Vec<QueryParam>,
    #[serde(default)]
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewEnvironmentInput {
    pub project_id: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateEnvironmentInput {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
}

/// Only the scopes that actually have a backing table today. `collection`/`folder` will be
/// added once those entities exist (README §41) — the `variables` table already reserves
/// `collection_id`/`folder_id` columns so that won't be a migration-breaking change.
/// `runtime` is deliberately absent: it is never persisted (see `resolver.rs`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VariableScope {
    Global,
    Environment,
    Request,
}

impl VariableScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            VariableScope::Global => "global",
            VariableScope::Environment => "environment",
            VariableScope::Request => "request",
        }
    }
}

/// Internal, full-fidelity representation — carries the real secret value. Never send this
/// directly to the frontend; use `VariableView` instead (README §41 "never expose secrets
/// unnecessarily to the frontend").
#[derive(Debug, Clone)]
pub struct Variable {
    pub id: String,
    pub scope: VariableScope,
    pub project_id: Option<String>,
    pub environment_id: Option<String>,
    pub request_id: Option<String>,
    pub key: String,
    pub value: String,
    pub enabled: bool,
    pub is_secret: bool,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Frontend-safe projection: secret values are masked unless explicitly revealed.
#[derive(Debug, Clone, Serialize)]
pub struct VariableView {
    pub id: String,
    pub scope: VariableScope,
    pub project_id: Option<String>,
    pub environment_id: Option<String>,
    pub request_id: Option<String>,
    pub key: String,
    pub value: String,
    pub enabled: bool,
    pub is_secret: bool,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Variable> for VariableView {
    fn from(v: Variable) -> Self {
        let value = if v.is_secret { "•".repeat(8) } else { v.value };
        VariableView {
            id: v.id,
            scope: v.scope,
            project_id: v.project_id,
            environment_id: v.environment_id,
            request_id: v.request_id,
            key: v.key,
            value,
            enabled: v.enabled,
            is_secret: v.is_secret,
            description: v.description,
            created_at: v.created_at,
            updated_at: v.updated_at,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewVariableInput {
    pub scope: VariableScope,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub environment_id: Option<String>,
    #[serde(default)]
    pub request_id: Option<String>,
    pub key: String,
    #[serde(default)]
    pub value: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub is_secret: bool,
    #[serde(default)]
    pub description: Option<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateVariableInput {
    pub id: String,
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub is_secret: Option<bool>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub clear_description: bool,
}

/// Partial update — every field except `id` is optional and, if omitted, preserves the
/// current value. `body`/`clear_body` are split because `body: None` is ambiguous between
/// "don't touch it" and "set it to null"; `clear_body: true` disambiguates the latter.
/// Metadata only — never carries the body (README §19/§23 lazy loading for responses).
#[derive(Debug, Clone, Serialize)]
pub struct ResponseSummary {
    pub id: String,
    pub request_id: String,
    pub status: u16,
    pub status_text: String,
    pub duration_ms: u64,
    pub body_size: u64,
    pub created_at: DateTime<Utc>,
}

/// Everything except the body — fetched when a response entry is opened, still without
/// pulling a potentially huge body into memory. Fetch the body separately via a dedicated
/// command (`get_response_body`) so the viewer can decide how much of it to actually read.
#[derive(Debug, Clone, Serialize)]
pub struct ResponseMeta {
    pub id: String,
    pub request_id: String,
    pub status: u16,
    pub status_text: String,
    pub headers: Vec<HeaderEntry>,
    pub duration_ms: u64,
    pub body_size: u64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResponseBodyPayload {
    /// Lossy UTF-8 decode — binary bodies are out of scope for the viewer today (README §19
    /// full media support is future work; see PROJECT_MAP known limitations).
    pub text: String,
    /// True when `text` was cut short of the real body (see `response_store` read cap).
    pub truncated: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateRequestInput {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub headers: Option<Vec<HeaderEntry>>,
    #[serde(default)]
    pub query_params: Option<Vec<QueryParam>>,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub clear_body: bool,
}
