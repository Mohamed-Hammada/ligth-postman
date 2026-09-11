use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Methods the request editor and HTTP engine support. CONNECT is deliberately excluded: it's
/// a proxy-tunnel-establishment method, not something a client can send to an arbitrary origin
/// — reqwest/hyper reject it before it reaches the wire (see
/// `http_engine::tests::connect_is_rejected_before_touching_the_network_not_silently_downgraded`),
/// so offering it in the picker would just be a button that always fails.
pub const VALID_METHODS: &[&str] = &[
    "GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS", "TRACE",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    /// The environment that should be auto-selected whenever this project is opened. `None`
    /// means "No Environment" — travels with the project (export/import, git sync), same as
    /// any other project-level setting.
    pub default_environment_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewProjectInput {
    pub name: String,
}

/// `name: None` means "leave unchanged" — omitted fields must never clobber existing data.
/// `default_environment_id: Some(id)` sets it; `clear_default_environment_id: true` explicitly
/// resets it to "No Environment" (the two are distinguished the same way request body/description
/// clearing already is elsewhere in this file — a bare `None` can't tell "omitted" from "null").
#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdateProjectInput {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub default_environment_id: Option<String>,
    #[serde(default)]
    pub clear_default_environment_id: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeaderEntry {
    pub key: String,
    pub value: String,
    pub enabled: bool,
    #[serde(default)]
    pub description: Option<String>,
}

/// Query parameters are kept separate from `url` (never baked in) so serialization is
/// runtime-only — a disabled param, or a param whose value resolves from a variable, must
/// be re-derivable every time without ever rewriting the stored request (README §41 style).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryParam {
    pub key: String,
    pub value: String,
    pub enabled: bool,
    #[serde(default)]
    pub description: Option<String>,
}

/// Where an API-key auth value gets placed on the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiKeyLocation {
    Header,
    Query,
}

/// Task-pack LP-0107 scope: None/Bearer/Basic/ApiKey. "Inheritance foundation" and an OAuth2
/// extension point are explicitly NOT included — there is no Collection/Folder entity yet for
/// a request to inherit auth *from*, and a fake `Inherit` variant that inherits from nothing
/// would be exactly the placeholder behavior this pack forbids. Add it when Collections exist.
/// Every field is stored as the raw template — `{{token}}` is resolved at send/codegen time
/// by `canonical_request::build`, same as headers/body/query params.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Auth {
    None,
    Bearer { token: String },
    Basic { username: String, password: String },
    ApiKey { key: String, value: String, location: ApiKeyLocation },
}

impl Default for Auth {
    fn default() -> Self {
        Auth::None
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormDataPart {
    pub key: String,
    pub value: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub is_file: bool,
    #[serde(default)]
    pub file_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UrlEncodedItem {
    pub key: String,
    pub value: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RequestBody {
    None,
    Raw {
        #[serde(default = "default_raw_content_type")]
        content_type: String,
        data: String,
    },
    #[serde(alias = "formdata", alias = "form-data")]
    FormData {
        items: Vec<FormDataPart>,
    },
    #[serde(alias = "urlencoded", alias = "x-www-form-urlencoded")]
    UrlEncoded {
        items: Vec<UrlEncodedItem>,
    },
    #[serde(rename = "graphql", alias = "graph_ql", alias = "graph_q_l")]
    GraphQL {
        query: String,
        #[serde(default)]
        variables: Option<String>,
    },
    Binary {
        #[serde(default)]
        file_path: Option<String>,
    },
}

fn default_raw_content_type() -> String {
    "application/json".to_string()
}

/// A single resolved multipart/form-data part, produced by `canonical_request::resolve_body`
/// and consumed by `http_engine::execute` to build a real `reqwest::multipart::Form` — the
/// canonical model carries structured parts here rather than a flattened string precisely
/// because multipart bodies (unlike raw/urlencoded/GraphQL) cannot be correctly represented as
/// one string: the boundary must be generated by the HTTP layer, and file parts need their
/// bytes read at send time, not baked into a template string ahead of time.
#[derive(Debug, Clone)]
pub enum ResolvedFormPart {
    Text { key: String, value: String },
    File { key: String, file_name: String, file_path: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RequestSettings {
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    #[serde(default)]
    pub follow_redirects: Option<bool>,
    #[serde(default)]
    pub max_redirects: Option<u32>,
    #[serde(default)]
    pub verify_ssl: Option<bool>,
    #[serde(default)]
    pub proxy_url: Option<String>,
    #[serde(default)]
    pub http_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cookie {
    pub id: String,
    pub project_id: String,
    pub domain: String,
    pub path: String,
    pub name: String,
    pub value: String,
    #[serde(default)]
    pub expires: Option<DateTime<Utc>>,
    #[serde(default)]
    pub secure: bool,
    #[serde(default)]
    pub http_only: bool,
    #[serde(default)]
    pub same_site: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewCookieInput {
    pub project_id: String,
    pub domain: String,
    #[serde(default = "default_cookie_path")]
    pub path: String,
    pub name: String,
    pub value: String,
    #[serde(default)]
    pub expires: Option<DateTime<Utc>>,
    #[serde(default)]
    pub secure: bool,
    #[serde(default)]
    pub http_only: bool,
    #[serde(default)]
    pub same_site: Option<String>,
}

fn default_cookie_path() -> String {
    "/".to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponseCookie {
    pub name: String,
    pub value: String,
    #[serde(default)]
    pub domain: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub expires: Option<String>,
    #[serde(default)]
    pub http_only: bool,
    #[serde(default)]
    pub secure: bool,
    #[serde(default)]
    pub same_site: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleResponse {
    pub id: String,
    pub request_id: String,
    pub name: String,
    pub status: u16,
    pub status_text: String,
    pub headers: Vec<HeaderEntry>,
    pub body: Option<String>,
    pub content_type: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewSampleResponseInput {
    pub request_id: String,
    pub name: String,
    pub status: u16,
    pub status_text: String,
    #[serde(default)]
    pub headers: Vec<HeaderEntry>,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub content_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateSampleResponseInput {
    pub id: String,
    pub name: String,
}

/// Lightweight row for lists — never carries headers/body (README §4/§20 lazy loading).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestSummary {
    pub id: String,
    pub project_id: String,
    #[serde(default)]
    pub folder_id: Option<String>,
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
    #[serde(default)]
    pub folder_id: Option<String>,
    pub name: String,
    pub method: String,
    pub url: String,
    pub headers: Vec<HeaderEntry>,
    pub query_params: Vec<QueryParam>,
    pub auth: Auth,
    pub body: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub settings: Option<RequestSettings>,
    #[serde(default)]
    pub pre_request_script: Option<String>,
    #[serde(default)]
    pub post_request_script: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct NewRequestInput {
    pub project_id: String,
    #[serde(default)]
    pub folder_id: Option<String>,
    pub name: String,
    pub method: String,
    pub url: String,
    #[serde(default)]
    pub headers: Vec<HeaderEntry>,
    #[serde(default)]
    pub query_params: Vec<QueryParam>,
    #[serde(default)]
    pub auth: Auth,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub settings: Option<RequestSettings>,
    #[serde(default)]
    pub pre_request_script: Option<String>,
    #[serde(default)]
    pub post_request_script: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// An environment plus the name of the project it happens to be stored under. Environment
/// selection is global (any project can select any environment — see `list_all_environments`);
/// `project_name` here is just display context in that picker, not an access restriction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentWithProject {
    pub id: String,
    pub project_id: String,
    pub project_name: String,
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
    pub is_local: bool,
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
    pub is_local: bool,
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
            is_local: v.is_local,
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
    pub is_local: bool,
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
    pub is_local: Option<bool>,
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

/// One row of the project-wide History screen — a response joined with its request's
/// name/method/url so the list is browsable without opening each request individually.
/// Still never carries the body (same lazy-loading rule as `ResponseSummary`).
#[derive(Debug, Clone, Serialize)]
pub struct ProjectHistoryEntry {
    pub id: String,
    pub request_id: String,
    pub request_name: String,
    pub method: String,
    pub url: String,
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
    pub content_type: Option<String>,
    pub cookies: Vec<ResponseCookie>,
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

#[derive(Debug, Clone, Default, Deserialize)]
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
    pub auth: Option<Auth>,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub clear_body: bool,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub clear_description: bool,
    #[serde(default)]
    pub settings: Option<RequestSettings>,
    #[serde(default)]
    pub clear_settings: bool,
    #[serde(default)]
    pub pre_request_script: Option<String>,
    #[serde(default)]
    pub clear_pre_request_script: bool,
    #[serde(default)]
    pub post_request_script: Option<String>,
    #[serde(default)]
    pub clear_post_request_script: bool,
    #[serde(default)]
    pub folder_id: Option<String>,
    /// Moves the request to the project's root, out of whatever folder it was in.
    #[serde(default)]
    pub clear_folder_id: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewFolderInput {
    pub project_id: String,
    pub name: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdateFolderInput {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
}
